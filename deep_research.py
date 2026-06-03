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
import requests
from bs4 import BeautifulSoup

def log_progress(log_file, message):
    timestamp = datetime.datetime.now().strftime("%H:%M:%S")
    formatted = f"[{timestamp}] {message}"
    print(formatted)
    if log_file:
        with open(log_file, "a", encoding="utf-8") as f:
            f.write(formatted + "\n")

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

def search_duckduckgo(query, limit=5):
    # Search DuckDuckGo HTML search page
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
            # ddg links are sometimes wrapped in /l/?kh=...&uddg=URL
            parsed = urllib.parse.urlparse(href)
            target_url = href
            if parsed.path == "/l/":
                qs = urllib.parse.parse_qs(parsed.query)
                if "uddg" in qs:
                    target_url = qs["uddg"][0]
            
            # Find snippet and title
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

def scrape_url(url):
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    }
    try:
        res = requests.get(url, headers=headers, timeout=15)
        res.raise_for_status()
        soup = BeautifulSoup(res.text, "html.parser")
        
        # Remove scripts, styles, nav elements
        for elem in soup(["script", "style", "nav", "footer", "header"]):
            elem.extract()
            
        text = soup.get_text(separator=" ")
        # Clean text
        text = re.sub(r'\s+', ' ', text).strip()
        
        # Truncate
        if len(text) > 8000:
            text = text[:8000] + "... [truncated]"
        return text
    except Exception as e:
        return f"Failed to scrape: {e}"

def generate_sub_queries(query, host, port, model, log_file):
    system_prompt = "You are a research query planner. Output only a JSON array of 3 distinct, search-friendly queries related to the topic."
    user_prompt = f"Topic to research: '{query}'\nGenerate 3 search engine queries to gather a broad base of information. Output as a JSON array of strings: [\"q1\", \"q2\", \"q3\"]."
    
    log_progress(log_file, "Planning research queries...")
    response = call_llm(host, port, model, system_prompt, user_prompt)
    
    # Try parsing JSON
    try:
        # Clean JSON markdown fences
        clean = response.strip().strip("`").strip()
        if clean.startswith("json"):
            clean = clean[4:].strip()
        queries = json.loads(clean)
        if isinstance(queries, list):
            return queries[:3]
    except Exception:
        pass
        
    # Fallback to regex split or simple queries
    queries = [f"{query} overview", f"{query} details", f"{query} latest news"]
    return queries

def generate_depth_queries(query, gathered_notes, host, port, model, log_file):
    system_prompt = "You are an analytical researcher. Based on the notes gathered so far, generate 2 specific follow-up search queries to deep-dive into unanswered questions or verify facts."
    user_prompt = f"Research Topic: {query}\n\nGathered Notes:\n{gathered_notes}\n\nOutput exactly a JSON array of 2 follow-up search queries: [\"q1\", \"q2\"]."
    
    log_progress(log_file, "Synthesizing phase 1 and planning deep-dive queries...")
    response = call_llm(host, port, model, system_prompt, user_prompt)
    
    try:
        clean = response.strip().strip("`").strip()
        if clean.startswith("json"):
            clean = clean[4:].strip()
        queries = json.loads(clean)
        if isinstance(queries, list):
            return queries[:2]
    except Exception:
        pass
        
    return [f"{query} specifications", f"{query} analysis"]

def main():
    parser = argparse.ArgumentParser(description="Multi-step Deep Research Agent")
    parser.add_argument("--query", required=True, help="Research query")
    parser.add_argument("--host", default="127.0.0.1", help="Llama server host")
    parser.add_argument("--port", type=int, default=8080, help="Llama server port")
    parser.add_argument("--model", default="", help="Model name")
    parser.add_argument("--output", required=True, help="Path to write the report")
    parser.add_argument("--log-file", help="Path to write progress logs")
    
    args = parser.parse_args()
    
    # Clear log file
    if args.log_file:
        with open(args.log_file, "w", encoding="utf-8") as f:
            f.write("")
            
    log_progress(args.log_file, f"Starting Deep Research: '{args.query}'")
    
    # Phase 1: Breadth Search Planning
    queries = generate_sub_queries(args.query, args.host, args.port, args.model, args.log_file)
    log_progress(args.log_file, f"Phase 1 sub-queries: {queries}")
    
    sources = []
    scraped_content = {}
    
    # Search and Crawl Phase 1
    for q in queries:
        log_progress(args.log_file, f"Searching for: '{q}'")
        search_results = search_duckduckgo(q, limit=3)
        for r in search_results:
            url = r["url"]
            if url not in scraped_content:
                log_progress(args.log_file, f"Found source: {r['title']} ({url})")
                sources.append(r)
                log_progress(args.log_file, f"Scraping text from: {url}")
                text = scrape_url(url)
                scraped_content[url] = {
                    "title": r["title"],
                    "content": text
                }
                
    # Phase 1 Synthesis
    notes_summary = ""
    for url, info in scraped_content.items():
        notes_summary += f"Source: {info['title']} ({url})\nContent excerpt: {info['content'][:800]}\n\n"
        
    # Phase 2: Depth Search Planning
    depth_queries = generate_depth_queries(args.query, notes_summary[:6000], args.host, args.port, args.model, args.log_file)
    log_progress(args.log_file, f"Phase 2 deep-dive queries: {depth_queries}")
    
    # Search and Crawl Phase 2
    for q in depth_queries:
        log_progress(args.log_file, f"Deep-dive searching for: '{q}'")
        search_results = search_duckduckgo(q, limit=2)
        for r in search_results:
            url = r["url"]
            if url not in scraped_content:
                log_progress(args.log_file, f"Found deep source: {r['title']} ({url})")
                sources.append(r)
                log_progress(args.log_file, f"Scraping text from: {url}")
                text = scrape_url(url)
                scraped_content[url] = {
                    "title": r["title"],
                    "content": text
                }
                
    # Compile All Context
    full_context = "RESEARCH SOURCES AND EXTRACTS:\n\n"
    for i, (url, info) in enumerate(scraped_content.items(), 1):
        full_context += f"[{i}] Title: {info['title']}\nURL: {url}\nContent:\n{info['content']}\n\n---\n\n"
        
    log_progress(args.log_file, "Synthesizing final research report...")
    
    system_prompt = (
        "You are an Elite Research Scientist. Synthesize all provided source extracts into a comprehensive, "
        "highly informative visual Markdown report.\n"
        "Requirements:\n"
        "1. Write clear, analytical sections with a logical flow.\n"
        "2. Add a comparison table or structured matrix comparing key elements.\n"
        "3. Provide a Mermaid diagram depicting the workflow, system design, or relationship map.\n"
        "4. Quote sources and provide inline citations referencing the sources.\n"
        "5. Output ONLY the markdown content of the report without any extra talking."
    )
    
    user_prompt = f"Research Query: {args.query}\n\n{full_context}\n\nWrite the final visual research report now:"
    
    report = call_llm(args.host, args.port, args.model, system_prompt, user_prompt, temperature=0.3)
    
    # Append sources section if not already present
    if "References" not in report and "Sources" not in report:
        report += "\n\n## Sources and References\n"
        for i, s in enumerate(sources, 1):
            report += f"- [{i}] [{s['title']}]({s['url']}) - {s['snippet']}\n"
            
    # Write to output file
    with open(args.output, "w", encoding="utf-8") as f:
        f.write(report)
        
    log_progress(args.log_file, "Deep Research completed successfully. Report saved.")
    print("FINISHED")

if __name__ == "__main__":
    main()
