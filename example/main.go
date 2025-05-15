package main

import (
	"fmt"
	"os"

	"github.com/anyproto/tantivy-go"
)

const NameBody = "body"
const NameId = "id"
const NameTitle = "title"

func main() {
	// Initialize the library
	err := tantivy_go.LibInit(true, true, "debug")
	if err != nil {
		fmt.Println("Failed to initialize library:", err)
		return
	}
	// Create schema builder
	builder, err := tantivy_go.NewSchemaBuilder()
	if err != nil {
		fmt.Println("Failed to create schema builder:", err)
		return
	}

	// Add fields to schema
	err = builder.AddTextField(
		NameTitle,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerEdgeNgram,
	)

	if err != nil {
		fmt.Println("Failed to add text field:", err)
		return
	}

	err = builder.AddTextField(
		NameId,
		true,
		false,
		false,
		tantivy_go.IndexRecordOptionBasic,
		tantivy_go.TokenizerRaw,
	)

	if err != nil {
		fmt.Println("Failed to add text field:", err)
		return
	}

	err = builder.AddTextField(
		NameBody,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerSimple,
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
	_ = os.RemoveAll("index_dir")
	index, err := tantivy_go.NewTantivyContextWithSchema("index_dir", schema)
	if err != nil {
		fmt.Println("Failed to create index:", err)
		return
	}
	defer index.Free()

	err = index.RegisterTextAnalyzerSimple(tantivy_go.TokenizerSimple, 40, tantivy_go.English)
	if err != nil {
		fmt.Println("Failed to register text analyzer:", err)
		return
	}

	err = index.RegisterTextAnalyzerEdgeNgram(tantivy_go.TokenizerEdgeNgram, 2, 4, 100)
	if err != nil {
		fmt.Println("Failed to register text analyzer:", err)
		return
	}

	err = index.RegisterTextAnalyzerRaw(tantivy_go.TokenizerRaw)
	if err != nil {
		fmt.Println("Failed to register text analyzer:", err)
		return
	}

	// Create document
	doc := tantivy_go.NewDocument()
	if doc == nil {
		fmt.Println("Failed to create document")
		return
	}

	// Add fields to document
	err = doc.AddField("Example Title", index, NameTitle)
	if err != nil {
		fmt.Println("Failed to add field to document:", err)
		return
	}

	err = doc.AddField("1", index, NameId)
	if err != nil {
		fmt.Println("Failed to add field to document:", err)
		return
	}

	err = doc.AddField("Example body content.", index, NameBody)
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
	sCtx := tantivy_go.NewSearchContextBuilder().
		SetQuery("body").
		SetDocsLimit(100).
		SetWithHighlights(true).
		AddFieldDefaultWeight(NameBody).
		Build()

	result, err := index.Search(sCtx)
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
		jsonStr, err := doc.ToJson(index, NameId, NameTitle, NameBody)
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
