package tantivy_go

type QueryType int

const (
	BoolQuery QueryType = iota
	PhraseQuery
	PhrasePrefixQuery
	TermPrefixQuery
	TermQuery
	EveryTermQuery
	OneOfTermQuery
)

type QueryModifier int

const (
	Must QueryModifier = iota
	Should
	MustNot
)

type FieldQuery struct {
	FieldIndex int     `json:"field_index"`
	TextIndex  int     `json:"text_index"`
	Boost      float64 `json:"boost"`
}

type QueryElement struct {
	Query     Query         `json:"query"`
	Modifier  QueryModifier `json:"query_modifier"`
	QueryType QueryType     `json:"query_type"`
}

type BooleanQuery struct {
	Subqueries []QueryElement `json:"subqueries"`
	Boost      float64        `json:"boost"`
}

type FinalQuery struct {
	Texts  []string      `json:"texts"`
	Fields []string      `json:"fields"`
	Query  *BooleanQuery `json:"query"`
}

type sharedStore struct {
	texts     map[string]int
	fields    map[string]int
	textList  []string
	fieldList []string
}

type QueryBuilder struct {
	store      *sharedStore
	subqueries []QueryElement
}

func NewQueryBuilder() *QueryBuilder {
	return &QueryBuilder{
		store: &sharedStore{
			texts:     make(map[string]int),
			fields:    make(map[string]int),
			textList:  []string{},
			fieldList: []string{},
		},
		subqueries: []QueryElement{},
	}
}

func (qb *QueryBuilder) NestedBuilder() *QueryBuilder {
	return &QueryBuilder{
		store:      qb.store,
		subqueries: []QueryElement{},
	}
}

func (qb *QueryBuilder) AddText(text string) int {
	if idx, exists := qb.store.texts[text]; exists {
		return idx
	}
	idx := len(qb.store.textList)
	qb.store.texts[text] = idx
	qb.store.textList = append(qb.store.textList, text)
	return idx
}

func (qb *QueryBuilder) AddField(field string) int {
	if idx, exists := qb.store.fields[field]; exists {
		return idx
	}
	idx := len(qb.store.fieldList)
	qb.store.fields[field] = idx
	qb.store.fieldList = append(qb.store.fieldList, field)
	return idx
}

func (qb *QueryBuilder) Query(modifier QueryModifier, field string, text string, queryType QueryType, boost float64) *QueryBuilder {
	textIndex := qb.AddText(text)
	fieldIndex := qb.AddField(field)
	qb.subqueries = append(qb.subqueries, QueryElement{
		Query: &FieldQuery{
			FieldIndex: fieldIndex,
			TextIndex:  textIndex,
			Boost:      boost,
		},
		Modifier:  modifier,
		QueryType: queryType,
	})
	return qb
}

func (qb *QueryBuilder) BooleanQuery(modifier QueryModifier, subBuilder *QueryBuilder, boost float64) *QueryBuilder {
	qb.subqueries = append(qb.subqueries, QueryElement{
		Query: &BooleanQuery{
			Subqueries: subBuilder.subqueries,
			Boost:      boost,
		},
		Modifier:  modifier,
		QueryType: BoolQuery,
	})
	return qb
}

func (qb *QueryBuilder) Build() FinalQuery {
	return FinalQuery{
		Texts:  qb.store.textList,
		Fields: qb.store.fieldList,
		Query: &BooleanQuery{
			Subqueries: qb.subqueries,
		},
	}
}

type Query interface {
	IsQuery()
}

func (fq *FieldQuery) IsQuery() {}

func (bq *BooleanQuery) IsQuery() {}
