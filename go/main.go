package main

import (
	"fmt"

	"github.com/anyproto/tantivy-go/go/tantivy"
)

const NameBody = "body"
const NameId = "id"
const NameTitle = "title"

func main() {
	// Initialize the library
	tantivy.LibInit("debug")
	// Create schema builder
	builder, err := tantivy.NewSchemaBuilder()
	if err != nil {
		fmt.Println("Failed to create schema builder:", err)
		return
	}

	// Add fields to schema
	err = builder.AddTextField(
		NameTitle,
		true,
		true,
		tantivy.IndexRecordOptionWithFreqsAndPositions,
		tantivy.TokenizerEdgeNgram,
	)

	if err != nil {
		fmt.Println("Failed to add text field:", err)
		return
	}

	err = builder.AddTextField(
		NameId,
		true,
		false,
		tantivy.IndexRecordOptionBasic,
		tantivy.TokenizerRaw,
	)

	if err != nil {
		fmt.Println("Failed to add text field:", err)
		return
	}

	err = builder.AddTextField(
		NameBody,
		true,
		true,
		tantivy.IndexRecordOptionWithFreqsAndPositions,
		tantivy.TokenizerSimple,
	)

	if err != nil {
		fmt.Println("Failed to add text field:", err)
		return
	}

	// Build schema
	schema, err := builder.BuildSchema()
	if err != nil {
		fmt.Println("Failed to build schema:", err)
		return
	}
	// Create index with schema
	index, err := tantivy.NewIndexWithSchema("index_dir", schema)
	if err != nil {
		fmt.Println("Failed to create index:", err)
		return
	}
	defer index.Free()

	err = index.RegisterTextAnalyzerSimple(tantivy.TokenizerSimple, 40, tantivy.English)
	if err != nil {
		fmt.Println("Failed to register text analyzer:", err)
		return
	}

	err = index.RegisterTextAnalyzerEdgeNgram(tantivy.TokenizerEdgeNgram, 2, 4, 100)
	if err != nil {
		fmt.Println("Failed to register text analyzer:", err)
		return
	}

	err = index.RegisterTextAnalyzerRaw(tantivy.TokenizerRaw)
	if err != nil {
		fmt.Println("Failed to register text analyzer:", err)
		return
	}

	// Create document
	doc := tantivy.NewDocument()
	if doc == nil {
		fmt.Println("Failed to create document")
		return
	}

	// Add fields to document
	err = doc.AddField(NameTitle, "Example Title", index)
	if err != nil {
		fmt.Println("Failed to add field to document:", err)
		return
	}

	err = doc.AddField(NameId, "1", index)
	if err != nil {
		fmt.Println("Failed to add field to document:", err)
		return
	}

	err = doc.AddField(NameBody, "Example body content.", index)
	if err != nil {
		fmt.Println("Failed to add field to document:", err)
		return
	}

	// Add document to index
	err = index.AddAndConsumeDocuments(doc)
	if err != nil {
		fmt.Println("Failed to add document:", err)
		return
	}

	// Search index
	result, err := index.Search("body", 100, NameBody)
	if err != nil {
		fmt.Println("Failed to search index:", err)
		return
	}
	defer result.Free()

	size, err := result.GetSize()
	if err != nil {
		fmt.Println("Failed to get search result size:", err)
		return
	}
	for next := range size {
		doc, err := result.Get(next)
		if err != nil {
			break
		}
		// Get JSON representation of the document
		jsonStr, err := doc.ToJson(schema, NameId, NameTitle, NameBody)
		if err != nil {
			fmt.Println("Failed to get document JSON:", err)
		} else {
			fmt.Println("Document JSON:")
			fmt.Println(jsonStr)
		}
		doc.Free()
	}

	docs, err := index.NumDocs()
	if err != nil {
		fmt.Println("Failed to get number of documents:", err)
		return
	}
	fmt.Println("Number of documents before:", docs)
	err = index.DeleteDocuments(NameId, "1")
	if err != nil {
		fmt.Println("Failed to delete document:", err)
		return
	}
	docs, err = index.NumDocs()
	fmt.Println("Number of documents after:", docs)
}
