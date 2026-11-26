# Filens

Navegador de arquivos para o terminal com pré-visualização.

## Pré-requisitos

- **Linux:** instale Rust e Cargo pelo [rustup](https://rust-lang.org/tools/install/) e tenha um compilador C com linker. No Debian ou Ubuntu, instale com `sudo apt install build-essential`.
- **Windows:** execute o instalador do [rustup](https://rust-lang.org/tools/install/) e siga as instruções para instalar as ferramentas de compilação C++ do Visual Studio com o Windows SDK.

Abra um novo terminal e confirme com `cargo --version`. Se for clonar o repositório, instale também o Git.

## Baixar o código

No terminal do Linux ou no PowerShell, rode:

```sh
git clone https://github.com/ngracioli/filens.git
cd filens
```

Execute os comandos Cargo abaixo na pasta que contém o `Cargo.toml`.

## Baixar só as dependências

```sh
cargo fetch --locked
```

O Cargo baixa as dependências do `Cargo.lock` sem compilar o Filens.

## Rodar pelo código-fonte

```sh
cargo run --locked -- .
```

O Cargo compila e abre o Filens na pasta atual. Troque `.` pelo caminho da pasta que quer navegar.

## Instalar no computador

O mesmo comando funciona no Linux e no Windows:

```sh
cargo install --path . --locked
```

Com a pasta de instalação no `PATH`, rode `filens .` de qualquer diretório.

- **Linux:** o binário fica em `~/.cargo/bin/filens` por padrão. Se usar Zsh e o comando não aparecer, adicione a linha abaixo ao fim de `~/.zshrc` e rode `source ~/.zshrc`.

  ```sh
  export PATH="$HOME/.cargo/bin:$PATH"
  ```

  Em Bash, use `~/.bashrc` e rode `source ~/.bashrc`. Confira com `command -v filens`.

- **Windows:** o executável fica em `%USERPROFILE%\.cargo\bin\filens.exe` por padrão. Se o comando não aparecer, adicione `%USERPROFILE%\.cargo\bin` ao `Path` nas variáveis de ambiente do usuário. Abra um novo PowerShell e confira com `Get-Command filens`.

## Atualizar ou reinstalar

Se as mudanças já estiverem no remoto da sua branch, atualize a cópia local com `git pull --ff-only`. Com o código novo na pasta do projeto, rode:

```sh
cargo install --path . --locked --force
```

O comando compila a cópia local e substitui o executável instalado, mesmo se a versão no `Cargo.toml` continuar igual.
