# Pontos Desktop - Rust + Tauri

Projeto desktop reconstruído em **Rust + Tauri 2 + Vue 3 + TypeScript + SQLite**, pensado como base funcional moderna para controle de ponto offline-first.

## O que já está implementado

- login local com sessão persistente
- usuário **master** inicial
- cadastro completo de **usuários**
- cadastro completo de **perfis de acesso**
- vínculo de usuários com múltiplas empresas
- vínculo de usuários com múltiplos perfis
- senha provisória e troca de senha por command
- dashboard com KPIs
- CRUDs de:
  - empresas usuárias
  - departamentos
  - funções
  - centros de custo
  - horários
  - escalas
  - equipamentos
  - eventos
  - justificativas
  - funcionários
  - jornadas de trabalho
- registro manual de batidas
- filtros de batidas
- exportação CSV de batidas
- apuração simples por período
- tratamento de ponto
- banco de horas
- fechamento mensal com espelho HTML
- fila local de sincronização
- auditoria de operações
- banco SQLite inicializado automaticamente

## Credenciais iniciais

- Login: `admin`
- Senha: `admin123`

Esse usuário nasce como **master_user = true**, com acesso total, perfil `Master` vinculado e acesso a todas as empresas cadastradas.

## Stack

- Frontend: Vue 3 + Vite + TypeScript + Pinia + Vue Router
- Desktop: Tauri 2
- Backend local: Rust
- Banco local: SQLite via rusqlite

## Estrutura

```text
src/
  config/
  layouts/
  pages/
  router/
  services/
  stores/
  styles.css

src-tauri/src/
  commands/
    access.rs
    auth.rs
    ...
  app_state.rs
  db.rs
  lib.rs
  main.rs
  migrations.rs
  models.rs
  security.rs
```

## Como rodar

### Desenvolvimento

```bash
npm install
npm run tauri:dev
```

### Build local

```bash
npm install
npm run tauri:build
```

## Fluxo de login e sessão

- o login gera um `session_token` salvo localmente no frontend
- a sessão é restaurada automaticamente ao abrir novamente a aplicação
- o backend mantém a sessão em `user_sessions`
- a validade padrão da sessão é de 7 dias
- o logout remove a sessão local e do banco

## Módulo de usuários e perfis

### Usuários
- nome, login, e-mail, telefone, cargo e observações
- senha inicial ou redefinição opcional na edição
- flag de **usuário master**
- flag de **administrador**
- flag de **senha provisória**
- status ativo/inativo
- vínculo com múltiplas empresas
- vínculo com múltiplos perfis
- bloqueio de login duplicado
- proteção contra exclusão do último usuário master
- proteção contra exclusão da própria conta master logada

### Perfis de acesso
- nome e descrição
- perfil master opcional
- status ativo/inativo
- matriz de permissões por recurso/ação
- contagem de usuários vinculados
- proteção contra exclusão quando houver usuários vinculados

## Banco local

O banco é criado automaticamente no diretório local de dados do usuário:

- Windows: `%LOCALAPPDATA%/pontos_desktop_tauri/pontos.db`
- Linux: `~/.local/share/pontos_desktop_tauri/pontos.db`
- macOS: `~/Library/Application Support/pontos_desktop_tauri/pontos.db`

## Build binário por plataforma

### Linux

Em Linux, o Tauri pode gerar AppImage, `.deb` e outros formatos conforme o bundler/plataforma. A documentação oficial recomenda usar Linux nativo, Docker ou GitHub Actions para esse build. citeturn277492view0turn106122search18

### Windows

Para gerar instalador/bundle Windows, o caminho oficial é rodar `tauri build` em Windows. O Tauri informa que `.msi` depende do WiX rodando em Windows, embora exista possibilidade de cross-compilação com NSIS e `cargo-xwin` em Linux/macOS com ressalvas. citeturn642469view2

### macOS

Este projeto foi configurado com **ad-hoc signing** (`bundle.macOS.signingIdentity = "-"`), útil quando você não quer usar assinatura Apple autenticada. A documentação do Tauri informa que esse modo ainda pode exigir que o usuário libere manualmente a aplicação em **Privacy & Security**, e que a distribuição direta fora da App Store normalmente requer code signing e notarização para evitar fricção de abertura. citeturn642469view1turn642469view0

## Workflow GitHub Actions

Foi incluído:

- `.github/workflows/build-desktop.yml`

Esse workflow usa `tauri-apps/tauri-action`, conforme o guia oficial do Tauri para GitHub Actions, para gerar artefatos de Linux, Windows e macOS. citeturn277492view0

## Scripts úteis

```bash
npm run tauri:dev
npm run tauri:build
npm run tauri:build:debug
npm run tauri:bundle:mac
```

## Observação importante sobre macOS sem assinatura Apple

Você pediu explicitamente para **não distribuir via loja** e **não depender de assinatura Apple**. O projeto foi configurado para **compilar e empacotar** no macOS sem certificado Apple usando assinatura ad-hoc. Isso ajuda principalmente em Apple Silicon, mas não elimina os avisos e liberações manuais do sistema operacional ao abrir uma aplicação baixada da Internet. citeturn642469view1
