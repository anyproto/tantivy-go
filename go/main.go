package main

import (
	"fmt"

	"github.com/anyproto/tantivy-go/go/tantivy"
)

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
	err = builder.AddTextField("title", true)
	if err != nil {
		fmt.Println("Failed to add text field:", err)
		return
	}

	err = builder.AddTextField("body", true)
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
	defer schema.Free()

	// Create index with schema
	index, err := tantivy.NewIndexWithSchema("index_dir", schema)
	if err != nil {
		fmt.Println("Failed to create index:", err)
		return
	}
	defer index.Free()

	// Create document
	doc := tantivy.NewDocument()
	if doc == nil {
		fmt.Println("Failed to create document")
		return
	}

	// Add fields to document
	err = doc.AddField("title", "Example Title", index)
	if err != nil {
		fmt.Println("Failed to add field to document:", err)
		return
	}

	err = doc.AddField("body", "Example body content.", index)
	if err != nil {
		fmt.Println("Failed to add field to document:", err)
		return
	}

	// Add document to index
	err = index.AddAndConsumeDocument(doc)
	if err != nil {
		fmt.Println("Failed to add document:", err)
		return
	}

	// Search index
	result, err := index.Search("body")
	if err != nil {
		fmt.Println("Failed to search index:", err)
		return
	}
	defer result.Free()

	// Iterate through search results
	for {
		doc, err := result.GetNext()
		if err != nil {
			break
		}
		// Get JSON representation of the document
		jsonStr, err := doc.ToJSON(schema)
		if err != nil {
			fmt.Println("Failed to get document JSON:", err)
		} else {
			fmt.Println("Document JSON:")
			fmt.Println(jsonStr)
		}
		doc.Free()
	}
}
