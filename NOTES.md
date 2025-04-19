# Notes about implementation

## Parser

The parser is implemented using tree-sitter - see the [grammar.js](https://github.com/ilya-klyuchnikov/tree-sitter-refal/blob/0.2.0/grammar.js).

`parser.rs` constructs the AST from the tree-sitter details.

## Runtime (VM)

References.
The runtime executes RASL (Refal Assembly).
RASL is described in the PhD thesis (chapter 3) and in the Preprint (chapter 3).
Here is the mapping of command names to the names used in the thesis and preprint
(with references to corresponding sections).

```rust
pub enum Command {
    // Checks that there is nothing between the border nodes.
    // PhD - 3.8 (ПРОВ)
    // Preprint - 3.8 (NIL)
    MatchEmpty,

    // Matches a symbol on the left border.
    // PhD - 3.8 (3HAЧ,S)
    // Preprint - 3.8 LSC(S)
    MatchSymbolL(String),
    // Matches a symbol on the right border.
    // PhD - 3.8 (ЗНАЧЯ,S)
    // Preprint - 3.8 RSC(S)
    MatchSymbolR(String),

    // Matches a (left) bracket on the left border.
    // PhD - 3.8 (СКОБ)
    // Preprint - 3.8 LB
    MatchStrBracketL,
    // Matches a (right) bracket on the right border.
    // PhD - 3.8 (СКОБЯ)
    // Preprint - 3.8 RB
    MatchStrBracketR,

    // Matches a symbol variable on the left border.
    // PhD - 3.9 (СИМ)
    // Preprint - 3.9 LS
    MatchSVarL,
    // Matches a symbol variable on the right border.
    // PhD - 3.9 (СИМЯ)
    // Preprint - 3.9 RS
    MatchSVarR,

    // Matches a bound (already projected) symbol variable on the left border.
    // PhD - 3.9 (СТС)
    // Preprint - 3.9 LSD
    MatchSVarLProj(usize),
    // Matches a bound (already projected) symbol variable on the right border.
    // PhD - 3.9 (СТСЯ)
    // Preprint - 3.9 RSD
    MatchSVarRProj(usize),

    // Matches a term variable on the left border.
    // PhD - 3.9 (ТЕРМ,N)
    // Preprint - 3.9 LW
    MatchTVarL,
    // Matches a term variable on the right border.
    // PhD - 3.9 (ТЕРМЯ,N)
    // Preprint - 3.9 RW
    MatchTVarR,

    // Matches a closed expression variable.
    // PhD - 3.9 (ЗАКР)
    // Preprint - 3.9 CE
    MatchEVar,
    // Prepares matching of an open expression variable.
    // PhD - 3.10 (ПУД)
    // Preprint - 3.10 (PLE)
    MatchEVarPrepare,
    // Lengthens the current open expression variable.
    // PhD - 3.10 (УД)
    // Preprint - 3.10 (LE)
    MatchEVarLengthen,

    // Matches a bound (already projected) expression variable on the left border.
    // PhD - 3.9 (CTB,N)
    // Preprint - 3.9 LED(N)
    MatchEVarLProj(usize),

    // Matches a bound (already projected) expression variable on the right border.
    // PhD - 3.9 (СТВЯ,N)
    // Preprint - 3.9 RED(N)
    MatchEVarRProj(usize),

    // Moves the left border to a corresponding projection.
    // PhD - 3.8 (УГР,N,M)
    // Preprint - 3.8 SB(N, M)
    MatchMoveBorderL(usize),

    // Moves the right border to a corresponding projection.
    // PhD - 3.8 (УГР,N,M)
    // Preprint - 3.8 SB(N, M)
    MatchMoveBorderR(usize),

    // Sets up a transition to the next sentence in the function.
    // PhD - 3.11 (УПЕР)
    // Preprint - 3.11 (SJUMP)
    SetupTransition(usize),
    // Truncates the jump stack (because lengthening cannot succeed).
    // PhD - 3.12 (КУД,N)
    // Preprint - 3.12 - EOE
    ConstrainLengthen(usize),

    // Starts rewriting mode.
    // Preprint - 3.15 (EOR)
    RewriteStart,
    // Inserts the left structural bracket.
    // PhD - 3.16 (BL)
    // Preprint - 3.16 (BL)
    InsertStrBracketL,
    // Inserts the right structural bracket.
    // PhD - 3.16 (BR)
    // Preprint - 3.16 (BR)
    InsertStrBracketR,

    // Inserts the left functional bracket.
    // PhD - 3.23
    InsertFunBracketL,
    // Inserts the right functional bracket.
    // PhD - 3.23
    InsertFunBracketR,

    // Inserts a symbol.
    // PhD - 3.16 (NS,S)
    // Preprint - 3.16 (NS)
    InsertSymbol(String),
    // Copies a symbol from the table of projections.
    // PhD - 3.17 (MULS,N)
    // Preprint - 3.18 (MULS,N)
    CopySymbol(usize),
    // Copies an expression from the table of projections.
    // PhD - 3.17 (MULE,N)
    // Preprint - 3.18 (MULE,N)
    CopyExpr(usize),
    // Transplants an object from the table of projections.
    // PhD - 3.18 (TPL,N,M)
    // Preprint - 3.19
    TransplantObject(usize),
    // Transplants an expression from the table of projections
    // PhD - 3.18 (TPL,N,M)
    // Preprint - 3.19
    TransplantExpr(usize),
    // Finalizes rewriting (cleans up).
    // PhD - 3.25
    // Preprint - 3.20
    RewriteFinalize,

    // Completes execution of the current sentence.
    // PhD - 3.25. Preprint - 3.20.
    Return,
}

```

