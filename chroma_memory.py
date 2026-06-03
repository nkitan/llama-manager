# /// script
# dependencies = [
#   "chromadb",
#   "fastembed",
# ]
# ///

import os
import sys
import json
import argparse
import datetime
import chromadb
from fastembed import TextEmbedding

DB_DIR = os.path.expanduser("~/.local/share/llama-manager/chroma_db")
os.makedirs(DB_DIR, exist_ok=True)

# Initialize ChromaDB persistent client
client = chromadb.PersistentClient(path=DB_DIR)
collection = client.get_or_create_collection(name="llama_manager_memories")

# Lazy loading of embedding model to speed up command-line arguments that don't need it
_embed_model = None
def get_embedding_model():
    global _embed_model
    if _embed_model is None:
        # fastembed uses ONNX runtime under the hood
        _embed_model = TextEmbedding()
    return _embed_model

def get_keywords(text):
    # Normalize and split into a set of unique words
    words = [w.strip(".,!?;:()\"'[]{}*+-/\\").lower() for w in text.split()]
    return {w for w in words if len(w) > 2}

def keyword_search(documents, query_text, limit=5):
    query_words = get_keywords(query_text)
    if not query_words:
        return []
    
    scored_docs = []
    for doc in documents:
        doc_text = doc.get("document", "")
        doc_words = get_keywords(doc_text)
        # Calculate overlap score
        matches = query_words.intersection(doc_words)
        score = len(matches)
        if score > 0:
            scored_docs.append((score, doc))
            
    # Sort by score descending
    scored_docs.sort(key=lambda x: x[0], reverse=True)
    return [doc for score, doc in scored_docs[:limit]]

def add_memory(text, scope, metadata_str=None):
    if not text.strip():
        return
        
    # Generate embedding
    model = get_embedding_model()
    embeddings = list(model.embed([text]))
    embedding = [float(x) for x in embeddings[0]]
    
    # Prep metadata
    metadata = {
        "scope": scope,
        "timestamp": datetime.datetime.now().isoformat(),
    }
    if metadata_str:
        try:
            extra = json.loads(metadata_str)
            metadata.update(extra)
        except Exception:
            pass
            
    doc_id = f"mem_{scope}_{datetime.datetime.now().timestamp()}_{hash(text) % 100000}"
    
    collection.add(
        ids=[doc_id],
        embeddings=[embedding],
        documents=[text],
        metadatas=[metadata]
    )
    print(json.dumps({"status": "success", "id": doc_id}))

def query_memories(query_text, scope=None, limit=5):
    # Fetch all documents to perform keyword matching as well
    # Since it is a local personal DB, fetching all elements or a large subset is fast
    all_data = collection.get()
    all_docs = []
    if all_data and "documents" in all_data and all_data["documents"]:
        for i in range(len(all_data["documents"])):
            doc_scope = all_data["metadatas"][i].get("scope")
            if scope and doc_scope != scope:
                continue
            all_docs.append({
                "id": all_data["ids"][i],
                "document": all_data["documents"][i],
                "metadata": all_data["metadatas"][i]
            })
            
    # 1. Keyword search
    kw_results = keyword_search(all_docs, query_text, limit=limit)
    
    # 2. Vector search
    model = get_embedding_model()
    embeddings = list(model.embed([query_text]))
    query_emb = [float(x) for x in embeddings[0]]
    
    where_filter = {}
    if scope:
        where_filter = {"scope": scope}
        
    vec_data = collection.query(
        query_embeddings=[query_emb],
        n_results=limit,
        where=where_filter if where_filter else None
    )
    
    vec_results = []
    if vec_data and "documents" in vec_data and vec_data["documents"]:
        docs = vec_data["documents"][0]
        ids = vec_data["ids"][0]
        metadatas = vec_data["metadatas"][0]
        # distances = vec_data.get("distances", [[]])[0]
        for i in range(len(docs)):
            vec_results.append({
                "id": ids[i],
                "document": docs[i],
                "metadata": metadatas[i],
                "search_type": "vector"
            })
            
    # Combine results (RRF / simple merge)
    combined = {}
    for item in vec_results:
        combined[item["id"]] = {
            "document": item["document"],
            "metadata": item["metadata"],
            "score_vec": 1,
            "score_kw": 0
        }
        
    for item in kw_results:
        doc_id = item["id"]
        if doc_id in combined:
            combined[doc_id]["score_kw"] = 1
        else:
            combined[doc_id] = {
                "document": item["document"],
                "metadata": item["metadata"],
                "score_vec": 0,
                "score_kw": 1
            }
            
    # Sort: overlap both first, then vector, then keyword
    merged_list = []
    for doc_id, val in combined.items():
        merged_list.append({
            "id": doc_id,
            "document": val["document"],
            "metadata": val["metadata"],
            "total_score": val["score_vec"] * 2 + val["score_kw"]
        })
        
    merged_list.sort(key=lambda x: x["total_score"], reverse=True)
    results = merged_list[:limit]
    
    print(json.dumps(results, indent=2))

def clear_db():
    client.delete_collection("llama_manager_memories")
    print(json.dumps({"status": "success"}))

def main():
    parser = argparse.ArgumentParser(description="ChromaDB Vector & Keyword retrieval service")
    subparsers = parser.add_subparsers(dest="command")
    
    add_parser = subparsers.add_parser("add")
    add_parser.add_argument("--text", required=True, help="Memory text to store")
    add_parser.add_argument("--scope", required=True, choices=["global", "project"], help="Scope of the memory")
    add_parser.add_argument("--metadata", help="JSON metadata string")
    
    query_parser = subparsers.add_parser("query")
    query_parser.add_argument("--text", required=True, help="Query text")
    query_parser.add_argument("--scope", choices=["global", "project"], help="Filter by scope")
    query_parser.add_argument("--limit", type=int, default=5, help="Max results")
    
    subparsers.add_parser("clear")
    
    args = parser.parse_args()
    
    if args.command == "add":
        add_memory(args.text, args.scope, args.metadata)
    elif args.command == "query":
        query_memories(args.text, args.scope, args.limit)
    elif args.command == "clear":
        clear_db()
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
