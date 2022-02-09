# Lexer

## Goal

### Performance
Performance should be achieved by reducing the total amount of if checks in any code path.

## Fuzzing

```bash
cd crates/lexer && cargo +nightly fuzz run lexer -- -only_ascii=1
```
