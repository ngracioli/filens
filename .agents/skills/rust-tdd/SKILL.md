---
name: rust-tdd
description: Use when a Rust change needs test-driven development, regression coverage for a bug, or a decision about whether and where to add a Cargo test for changed behavior.
---

# TDD em Rust

Proteja comportamentos que uma regressão plausível quebraria. Antes de editar, leia o contrato, os chamadores, os testes existentes e as instruções do projeto.

## Escolha o que testar

- Descreva a falha concreta que o teste pegaria e sua consequência. Priorize bugs reproduzíveis, regras de negócio, limites com comportamento distinto e erros que precisam preservar dados, estado ou compatibilidade.
- Reaproveite um teste existente se ele já detecta a regressão. Rode-o primeiro. Preserve as verificações do contrato que continuam válidas e atualize apenas expectativas que a mudança tornou obsoletas.
- Mudanças de formatação, getters triviais, detalhes internos e refactors cobertos pela suíte geralmente dispensam testes novos. Em refactor sem cobertura de um contrato de risco, primeiro registre o comportamento atual com um teste de caracterização que passe.
- Use o menor caminho de execução que ainda reproduz o risco.

| Risco | Onde testar |
| --- | --- |
| Regra local, parser ou transição de estado | Módulo `#[cfg(test)]` próximo ao código, com `#[test]`. |
| API pública, interação entre componentes ou contrato da CLI | `tests/`, pela API pública ou pelo binário via `env!("CARGO_BIN_EXE_nome")`, usando o nome real do binário. |
| Exemplo de uso que usuários copiam | Doctest no rustdoc da biblioteca. |
| Renderização ou interação de terminal | Backend em memória para layout e texto. Reserve teste com pseudoterminal para comportamento do terminal real. |

## Vermelho, verde, refatoração

1. **Vermelho.** Escreva ou adapte um teste para o próximo comportamento relevante. Rode só o caso com `cargo test --locked <filtro>` quando o projeto versionar o lockfile. Confirme pelo nome que o caso rodou e falhou pela expectativa em questão. Um filtro que encontra zero testes não comprova a falha. Corrija erros do fixture e do harness antes de interpretar o resultado. Se falta uma API nova, faça o mínimo para compilar e observe a falha de comportamento.
2. **Verde.** Implemente a menor mudança que satisfaz o contrato. Confirme que o mesmo caso foi executado e passou.
3. **Refatoração.** Simplifique código e teste quando houver ganho concreto. Rode a suíte pertinente e as verificações do projeto. Informe quais casos rodaram e qualquer etapa que ficou sem comprovação.

## Faça o teste durar

- Dê ao teste um nome que descreva o resultado esperado. Compare valores, variantes de erro ou estado observável. Verifique texto literal, ordem e chamadas internas apenas quando forem parte do contrato.
- Use fixtures pequenas e independentes. Para I/O, crie um diretório temporário próprio por teste e reaproveite os recursos do projeto. Configure diretório e ambiente no subprocesso quando necessário. Evite estado global, esperas fixas e dependência da ordem de execução.
- Comece com entradas representativas. Use tabela se os casos exercem a mesma regra, testes de propriedade quando uma invariável atravessa muitas entradas e snapshots somente para saída extensa cujo formato faz parte do contrato.

Exemplo. Para corrigir uma prévia que corta UTF-8 no limite de leitura, use bytes que terminem no meio de um caractere. Verifique o prefixo válido e o indicador de truncamento pela entrada normal da prévia. Inclua outro caso se bytes inválidos antes do limite tiverem tratamento distinto.

## Fontes

- [Martin Fowler sobre Red–Green–Refactor](https://martinfowler.com/bliki/TestDrivenDevelopment.html)
- [Rust Book sobre organização de testes](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust Book sobre execução paralela](https://doc.rust-lang.org/book/ch11-02-running-tests.html)
- [Cargo Book sobre filtros, targets e doctests](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
