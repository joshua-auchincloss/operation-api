hljs.registerLanguage(
  "pld",
  (function () {
    "use strict";
    return function (e) {
      return {
        name: "PLD",
        aliases: ["protocol"],
        keywords: {
          keyword:
            "schema namespace use struct enum type oneof error operation input output",
          built_in:
            "i8 i16 i32 i64 u8 u16 u32 u64 f16 f32 f64 bool str complex datetime binary never",
          literal: "true false",
        },
        contains: [
          e.C_LINE_COMMENT_MODE,
          e.C_BLOCK_COMMENT_MODE,
          {
            className: "meta",
            begin: /#!?\[/,
            end: /\]/,
            contains: [
              { className: "meta-string", begin: /"/, end: /"/ },
              e.C_NUMBER_MODE,
            ],
          },
          {
            begin: /\b(namespace)\s+/,
            beginScope: "keyword",
            end: /;/,
            contains: [
              {
                className: "title.class",
                match: /[a-z_][a-z0-9_]*(?:::[a-z_][a-z0-9_]*)*/,
              },
            ],
          },
          {
            begin: /\b(use)\s+/,
            beginScope: "keyword",
            end: /;/,
            contains: [
              {
                className: "title.class",
                match: /[a-z_][a-z0-9_]*(?:::[a-z_][a-z0-9_]*)*/,
              },
            ],
          },
          {
            className: "keyword",
            begin: /\b(struct|enum|type|oneof|error|operation)\s+/,
            end: /(?=[{=;])/,
            contains: [
              {
                className: "title.class",
                match: /[A-Z][A-Za-z0-9_]*/,
                relevance: 0,
              },
            ],
            relevance: 10,
          },
          e.QUOTE_STRING_MODE,
          {
            className: "number",
            variants: [
              { begin: /\b\d+\.\d+([eE][+-]?\d+)?\b/ },
              { begin: /\b0[xX][0-9a-fA-F]+\b/ },
              { begin: /\b0[oO][0-7]+\b/ },
              { begin: /\b0[bB][01]+\b/ },
              { begin: /\b\d+\b/ },
            ],
            relevance: 0,
          },
          {
            className: "type",
            match: /\b[A-Z][A-Za-z0-9_]*(?:::[A-Z][A-Za-z0-9_]*)*/,
          },
          {
            className: "type",
            begin: /\[/,
            end: /\]/,
            contains: ["self"],
          },
          { className: "variable", match: /\b[a-z_][a-z0-9_]*\s*(?=:)/ },
          {
            className: "title.function",
            match: /\b[A-Z][a-zA-Z0-9_]*(?=\()/,
          },
          {
            className: "operator",
            match: /->|::|[|&]|\?|!/,
            relevance: 0,
          },
        ],
      };
    };
  })(),
);
