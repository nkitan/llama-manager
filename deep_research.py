# /// script
# dependencies = [
#   "requests",
#   "beautifulsoup4",
# ]
# ///

import os
import sys
import json
import urllib.parse
import argparse
import datetime
import re
import hashlib
import shutil
import subprocess
import requests
from bs4 import BeautifulSoup

def log_progress(log_file, message):
    timestamp = datetime.datetime.now().strftime("%H:%M:%S")
    formatted = f"[{timestamp}] {message}"
    print(formatted)
    if log_file:
        try:
            with open(log_file, "a", encoding="utf-8") as f:
                f.write(formatted + "\n")
        except Exception as e:
            print(f"Error writing to log file: {e}", file=sys.stderr)

def call_llm(host, port, model, system_prompt, user_prompt, temperature=0.2):
    url = f"http://{host}:{port}/v1/chat/completions"
    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": temperature
    }
    try:
        res = requests.post(url, json=payload, timeout=60)
        res.raise_for_status()
        res_json = res.json()
        return res_json["choices"][0]["message"]["content"]
    except Exception as e:
        print(f"Error calling LLM at {url}: {e}", file=sys.stderr)
        return ""

def search_duckduckgo(query, limit=4):
    url = f"https://html.duckduckgo.com/html/?q={urllib.parse.quote(query)}"
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    }
    results = []
    try:
        res = requests.get(url, headers=headers, timeout=15)
        res.raise_for_status()
        soup = BeautifulSoup(res.text, "html.parser")
        for a in soup.find_all("a", class_="result__url"):
            href = a.get("href", "")
            parsed = urllib.parse.urlparse(href)
            target_url = href
            if parsed.path == "/l/":
                qs = urllib.parse.parse_qs(parsed.query)
                if "uddg" in qs:
                    target_url = qs["uddg"][0]
            
            title = ""
            snippet = ""
            parent = a.find_parent("div", class_="links_main")
            if parent:
                title_elem = parent.find("a", class_="result__a")
                if title_elem:
                    title = title_elem.get_text(strip=True)
                snippet_elem = parent.find_next_sibling("div", class_="result__snippet")
                if snippet_elem:
                    snippet = snippet_elem.get_text(strip=True)
            
            if target_url and title:
                results.append({
                    "title": title,
                    "url": target_url,
                    "snippet": snippet
                })
                if len(results) >= limit:
                    break
    except Exception as e:
        print(f"Search error for query '{query}': {e}", file=sys.stderr)
    return results

def search_searxng(searxng_url, query, limit=4):
    url = f"{searxng_url.rstrip('/')}/search?q={urllib.parse.quote(query)}&format=json"
    results = []
    try:
        res = requests.get(url, timeout=15)
        res.raise_for_status()
        res_json = res.json()
        if "results" in res_json:
            for r in res_json["results"][:limit]:
                results.append({
                    "title": r.get("title", ""),
                    "url": r.get("url", ""),
                    "snippet": r.get("content", r.get("snippet", ""))
                })
    except Exception as e:
        print(f"SearXNG search error: {e}", file=sys.stderr)
    return results

def scrape_url(url):
    harness_path = shutil.which("browser-harness")
    camofox_path = shutil.which("camofox")
    cloak_path = shutil.which("cloakbrowser")

    if harness_path:
        try:
            res = subprocess.run([harness_path, "--url", url], capture_output=True, text=True, timeout=30)
            if res.returncode == 0 and res.stdout.strip():
                return res.stdout
        except Exception:
            pass
    if camofox_path:
        try:
            res = subprocess.run([camofox_path, "--url", url, "--text-only"], capture_output=True, text=True, timeout=30)
            if res.returncode == 0 and res.stdout.strip():
                return res.stdout
        except Exception:
            pass
    if cloak_path:
        try:
            res = subprocess.run([cloak_path, "--url", url, "--text-only"], capture_output=True, text=True, timeout=30)
            if res.returncode == 0 and res.stdout.strip():
                return res.stdout
        except Exception:
            pass

    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    }
    try:
        res = requests.get(url, headers=headers, timeout=15)
        res.raise_for_status()
        soup = BeautifulSoup(res.text, "html.parser")
        
        for elem in soup(["script", "style", "nav", "footer", "header"]):
            elem.extract()
            
        text = soup.get_text(separator=" ")
        text = re.sub(r'\s+', ' ', text).strip()
        
        if len(text) > 12000:
            text = text[:12000] + "... [truncated]"
        return text
    except Exception as e:
        return f"Failed to scrape: {e}"

class TaskNode:
    def __init__(self, task_id, query, dependencies=None):
        self.id = task_id
        self.query = query
        self.dependencies = dependencies or []
        self.status = "pending"  # pending, running, completed, failed
        self.results = []

def parse_json_safely(text):
    try:
        clean = text.strip().strip("`").strip()
        if clean.startswith("json"):
            clean = clean[4:].strip()
        return json.loads(clean)
    except Exception:
        # Simple fallback parsing using regex
        try:
            m = re.search(r'\[\s*".*?"\s*\]', text, re.DOTALL)
            if m:
                return json.loads(m.group(0))
        except Exception:
            pass
        return None

def main():
    parser = argparse.ArgumentParser(description="Systems-Grade Deep Research Engine")
    parser.add_argument("--query", required=True, help="Research query")
    parser.add_argument("--host", default="127.0.0.1", help="Llama server host")
    parser.add_argument("--port", type=int, default=8080, help="Llama server port")
    parser.add_argument("--model", default="", help="Model name")
    parser.add_argument("--output", required=True, help="Path to write the report")
    parser.add_argument("--log-file", help="Path to write progress logs")
    parser.add_argument("--searxng-url", help="SearXNG URL")
    
    args = parser.parse_args()
    
    if args.log_file:
        try:
            with open(args.log_file, "w", encoding="utf-8") as f:
                f.write("")
        except Exception:
            pass
            
    log_progress(args.log_file, "🚀 Dynamic Deep Orchestration initialized.")
    log_progress(args.log_file, f"Query Target: '{args.query}'")
    
    # ── 1. External Knowledge Buffer Setup ──────────────────────────────────
    output_dir = os.path.dirname(args.output)
    buffer_dir = os.path.join(output_dir, "knowledge_buffer")
    os.makedirs(buffer_dir, exist_ok=True)
    log_progress(args.log_file, f"📁 External Knowledge Buffer linked at: {buffer_dir}")
    
    # ── 2. Task DAG Construction ─────────────────────────────────────────────
    log_progress(args.log_file, "🕸️ Designing Dynamic Task Dependency Graph (DAG)...")
    planner_prompt = (
        "You are an Elite Research Architect. Decompose the research topic into a Directed Acyclic Graph (DAG) "
        "of 3 distinct search-oriented sub-tasks. Each task must have a unique ID, a specific search query, "
        "and list any dependencies on other task IDs. Output ONLY a valid JSON object in this format:\n"
        "{\n"
        "  \"tasks\": [\n"
        "    {\"id\": \"t1\", \"query\": \"query matching first subtopic\", \"dependencies\": []},\n"
        "    {\"id\": \"t2\", \"query\": \"query matching second subtopic\", \"dependencies\": [\"t1\"]},\n"
        "    {\"id\": \"t3\", \"query\": \"query matching third subtopic\", \"dependencies\": []}\n"
        "  ]\n"
        "}"
    )
    user_prompt = f"Topic to research: '{args.query}'\nGenerate the research task DAG:"
    
    planner_res = call_llm(args.host, args.port, args.model, planner_prompt, user_prompt)
    dag_json = parse_json_safely(planner_res)
    
    tasks = {}
    if dag_json and "tasks" in dag_json:
        for t_data in dag_json["tasks"]:
            t_id = t_data.get("id")
            query = t_data.get("query")
            deps = t_data.get("dependencies", [])
            tasks[t_id] = TaskNode(t_id, query, deps)
            log_progress(args.log_file, f"  └─ Task [{t_id}]: '{query}' (dependencies: {deps})")
    else:
        # Fallback DAG
        tasks["t1"] = TaskNode("t1", f"{args.query} core concepts", [])
        tasks["t2"] = TaskNode("t2", f"{args.query} architecture and components", ["t1"])
        tasks["t3"] = TaskNode("t3", f"{args.query} industry applications case studies", [])
        log_progress(args.log_file, "  └─ [Fallback DAG loaded due to parser layout mismatch]")

    # ── 3. DAG Execution Loop with Dynamic Rewiring ──────────────────────────
    executed_count = 0
    scraped_sources = []
    scraped_urls = set()
    
    while any(t.status == "pending" for t in tasks.values()) and executed_count < 10:
        # Find tasks that have all dependencies met
        runnable = []
        for t_id, task in tasks.items():
            if task.status == "pending":
                deps_met = all(tasks[dep].status == "completed" for dep in task.dependencies if dep in tasks)
                if deps_met:
                    runnable.append(task)
                    
        if not runnable:
            # Break deadlocks
            for t_id, task in tasks.items():
                if task.status == "pending":
                    runnable.append(task)
                    break
                    
        for task in runnable:
            task.status = "running"
            log_progress(args.log_file, f"🔍 Executing Task [{task.id}]: '{task.query}'")
            
            # Execute search
            if hasattr(args, "searxng_url") and args.searxng_url:
                search_results = search_searxng(args.searxng_url, task.query, limit=3)
            else:
                search_results = search_duckduckgo(task.query, limit=3)
            if not search_results:
                log_progress(args.log_file, f"⚠️ No search results for Task [{task.id}]. Rewiring graph...")
                task.status = "failed"
                # Dynamic Rewiring: Spin up a broader replacement task
                new_id = f"t_rewire_{len(tasks) + 1}"
                broader_query = f"{task.query} alternative terms"
                tasks[new_id] = TaskNode(new_id, broader_query, [])
                log_progress(args.log_file, f"  └─ Added broad fallback Task [{new_id}]: '{broader_query}'")
                continue
                
            task_files = []
            for r in search_results:
                url = r["url"]
                if url in scraped_urls:
                    continue
                scraped_urls.add(url)
                
                log_progress(args.log_file, f"📥 Fetching & Scrapesing: {r['title']} ({url})")
                text_content = scrape_url(url)
                
                # External Knowledge Buffering: stream scraped content to disk
                url_hash = hashlib.md5(url.encode('utf-8')).hexdigest()[:8]
                buf_filename = f"src_{task.id}_{url_hash}.txt"
                buf_filepath = os.path.join(buffer_dir, buf_filename)
                try:
                    with open(buf_filepath, "w", encoding="utf-8") as bf:
                        bf.write(f"TITLE: {r['title']}\nURL: {url}\nDATE: {datetime.date.today()}\n\n{text_content}")
                    task_files.append(buf_filepath)
                    scraped_sources.append({"title": r["title"], "url": url, "snippet": r["snippet"], "file": buf_filepath})
                except Exception as e:
                    log_progress(args.log_file, f"Error saving external buffer: {e}")
                    
            task.status = "completed"
            task.results = task_files
            executed_count += 1
            
            # Dynamic Graph Mutation Check
            # Ask LLM if the gathered sources for this task reveal gaps requiring follow-up query
            if task_files:
                log_progress(args.log_file, f"🧬 Inspecting Task [{task.id}] knowledge density...")
                source_excerpts = ""
                for s in scraped_sources[-3:]:
                    source_excerpts += f"Title: {s['title']}\nSnippet: {s['snippet']}\n\n"
                    
                gap_prompt = (
                    "You are an Adversarial Audit Planner. Analyze the research query and current search results. "
                    "Determine if there are crucial gaps, missing details, or conflicting facts that require an "
                    "additional search task. If yes, output exactly a JSON object in this format:\n"
                    "{\"gap_detected\": true, \"query\": \"the targeted follow-up query\", \"reason\": \"description of gap\"}\n"
                    "If no gap exists or the data is sufficient, output: {\"gap_detected\": false}"
                )
                gap_user = f"Primary query: {args.query}\nSub-task query: {task.query}\nResults:\n{source_excerpts}"
                gap_res = call_llm(args.host, args.port, args.model, gap_prompt, gap_user)
                gap_json = parse_json_safely(gap_res)
                
                if gap_json and gap_json.get("gap_detected") and len(tasks) < 6:
                    new_q = gap_json.get("query")
                    new_id = f"t_gap_{len(tasks) + 1}"
                    tasks[new_id] = TaskNode(new_id, new_q, [task.id])
                    log_progress(args.log_file, f"  └─ Dynamic DAG Mutation: Spawning Task [{new_id}] due to gap: '{new_q}'")

    log_progress(args.log_file, "✅ Task graph execution complete. Buffers filled.")

    # ── 4. Proposer & Adversarial Auditor Loop ─────────────────────────────
    log_progress(args.log_file, "🛡️ Initializing Proposer & Adversarial Auditor collaboration...")
    
    # Compile index and short summaries for context
    buffer_summaries = ""
    for idx, s in enumerate(scraped_sources, 1):
        buffer_summaries += f"[{idx}] Title: {s['title']}\nURL: {s['url']}\nAbstract: {s['snippet']}\nLocal Buffer: {os.path.basename(s['file'])}\n\n"

    # We split synthesis into steps to prevent context overflow and generate grounded drafts
    log_progress(args.log_file, "✍️ Proposer generating initial draft...")
    proposer_system = (
        "You are an Elite Research Proposer. Write a highly detailed, comprehensive visual Markdown report "
        "covering the research topic. Incorporate a logical section structure, comparison tables, and a Mermaid diagram. "
        "Use the provided sources index and abstract summaries to ground your statements. Cite them as [idx]. "
        "Do not invent facts or URLs. Output ONLY the raw Markdown report."
    )
    proposer_user = f"Research Query: {args.query}\n\nSource Summaries:\n{buffer_summaries}\n\nDraft report:"
    draft = call_llm(args.host, args.port, args.model, proposer_system, proposer_user, temperature=0.3)
    
    # Audit Loop (Maximum 2 iterations of critique and refinement)
    for iteration in range(1, 3):
        log_progress(args.log_file, f"🔍 Adversarial Auditor auditing Draft (Iteration {iteration})...")
        
        # Read exact paragraphs from buffer to cross-check if needed, or ask the LLM to inspect grounding
        auditor_system = (
            "You are an Adversarial Auditor. Your job is to check the research draft for hallucinations, "
            "unverified claims, fabricated citations, or weak source references. Compare the assertions in the draft "
            "with the provided source summaries. Write a highly critical audit report detailing: "
            "1. Specific claims that are not grounded in the sources. "
            "2. Fabricated or suspicious citations. "
            "3. Missing perspectives or weak URLs. "
            "If the draft is grounded and accurate, write exactly: {\"approved\": true}. "
            "Otherwise, output a JSON object: {\"approved\": false, \"critique\": \"your detailed critique here\"}"
        )
        auditor_user = f"Research Draft:\n{draft}\n\nSource Summaries:\n{buffer_summaries}\n\nAudit Results:"
        audit_res = call_llm(args.host, args.port, args.model, auditor_system, auditor_user, temperature=0.1)
        audit_json = parse_json_safely(audit_res)
        
        if audit_json and audit_json.get("approved"):
            log_progress(args.log_file, "🛡️ Auditor status: APPROVED. No grounding errors detected.")
            break
        else:
            critique = audit_json.get("critique") if audit_json else "General grounding check failed."
            log_progress(args.log_file, f"❌ Auditor critique emitted: {critique}")
            log_progress(args.log_file, "🔄 Proposer refining draft based on auditor critique...")
            
            refiner_system = (
                "You are an Elite Research Proposer. Revise your research draft based on the Auditor's critique. "
                "Remove any ungrounded assertions, verify your citations and URLs, and ensure absolute grounding "
                "against the provided source summaries. Output ONLY the revised visual Markdown report."
            )
            refiner_user = f"Original Draft:\n{draft}\n\nAuditor Critique:\n{critique}\n\nSource Summaries:\n{buffer_summaries}\n\nRevised report:"
            draft = call_llm(args.host, args.port, args.model, refiner_system, refiner_user, temperature=0.2)

    # ── 5. Final Compilation & Synthesis ────────────────────────────────────
    log_progress(args.log_file, "📝 Synthesizing final visual research report...")
    
    # Append sources section
    if "References" not in draft and "Sources" not in draft:
        draft += "\n\n## Sources and References\n"
        for idx, s in enumerate(scraped_sources, 1):
            draft += f"- [{idx}] [{s['title']}]({s['url']}) - {s['snippet']}\n"
            
    # Write to output file
    try:
        with open(args.output, "w", encoding="utf-8") as f:
            f.write(draft)
        log_progress(args.log_file, "🎉 Deep Research completed successfully. Report saved.")
        print("FINISHED")
    except Exception as e:
        log_progress(args.log_file, f"Error saving final report: {e}")
        print("ERROR")

if __name__ == "__main__":
    main()
