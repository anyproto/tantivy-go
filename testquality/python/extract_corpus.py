import csv

import datasets

lang='en' # or any of the 16 languages
miracl_corpus = datasets.load_dataset('miracl/miracl-corpus', lang)['train']

with open("dataset.tsv", mode="w", encoding="utf-8", newline="") as file:
    writer = csv.writer(file, delimiter="\t")

    writer.writerow(["docid", "title", "text"])

    for doc in miracl_corpus:
        writer.writerow([doc['docid'], doc['title'], doc['text']])