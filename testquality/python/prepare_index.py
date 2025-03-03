import csv
import json
import random
import sys

queries_file = "./miracl/miracl-v1.0-en/topics/topics.miracl-v1.0-en-dev.tsv"
relevance_file = "./miracl/miracl-v1.0-en/qrels/qrels.miracl-v1.0-en-dev.tsv"
documents_file = "dataset.tsv"


documents = {}
queries = {}
relevance = {}
all_relevant_doc_ids = set()

with open(queries_file, "r", encoding="utf-8") as query_file:
    reader = csv.reader(query_file, delimiter="\t")
    for row in reader:
        query_id, query_text = row
        queries[query_id] = query_text

sampled_query_ids = queries.keys()
sampled_queries = {query_id: queries[query_id] for query_id in sampled_query_ids}

print("Collect relevant docs for the query...")
with open(relevance_file, "r", encoding="utf-8") as rel_file:
    reader = csv.reader(rel_file, delimiter="\t")
    for row in reader:
        query_id, _, doc_id, _ = row
        if query_id in sampled_query_ids:
            if query_id not in relevance:
                relevance[query_id] = []
            relevance[query_id].append(doc_id)
            all_relevant_doc_ids.add(doc_id)

print("Prepare output data...")
documents_json = [{"id": doc_id, "title": doc["title"], "body": doc["body"]} for doc_id, doc in documents.items()]

queries_json = []
for query_id, query_text in sampled_queries.items():
    if query_id in relevance:
        queries_json.append({
            "query": query_text,
            "relevant_docs": relevance[query_id]
        })

print("Save data in JSON format...")
with open("../documents.json", "w", encoding="utf-8") as doc_file:
    json.dump(documents_json, doc_file, indent=2, ensure_ascii=False)

with open("../queries.json", "w", encoding="utf-8") as query_file:
    json.dump(queries_json, query_file, indent=2, ensure_ascii=False)

print("Dataset has been created successfully!")
