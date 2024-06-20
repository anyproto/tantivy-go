use tantivy::tokenizer::*;
use tantivy::tokenizer::Token;

#[derive(Clone)]
pub struct EdgeNgramTokenizer {
    min_gram: usize,
    max_gram: usize,
    limit: usize,
}

impl EdgeNgramTokenizer {
    pub fn new(min_gram: usize,
               max_gram: usize,
               limit: usize,
    ) -> EdgeNgramTokenizer {
        EdgeNgramTokenizer {
            min_gram,
            max_gram,
            limit,
        }
    }
}

impl Tokenizer for EdgeNgramTokenizer {
    type TokenStream<'a> = BoxTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> BoxTokenStream<'a> {
        let text = if text.len() < self.limit { text } else { text.split_at(self.limit).0 };
        let mut tokens = Vec::new();

        for word in text.split_whitespace() {
            let chars: Vec<char> = word.chars().collect();
            for n in self.min_gram..=self.max_gram {
                if n <= chars.len() {
                    let text: String = chars[0..n].iter().collect();
                    tokens.push(Token {
                        offset_from: 0,
                        offset_to: n,
                        position: 0,
                        text,
                        position_length: 1,
                    });
                }
            }
        }

        BoxTokenStream::new(VecTokenStream { tokens, index: 0 })
    }
}

struct VecTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream for VecTokenStream {
    fn advance(&mut self) -> bool {
        if self.index < self.tokens.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.index - 1]
    }
}