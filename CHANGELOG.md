## [1.16.5](https://github.com/wkarts/argws-ponto-manager/compare/v1.16.4...v1.16.5) (2026-04-05)


### Bug Fixes

* **cartao-ponto:** corrige renderização das guias e largura homogênea ([633da88](https://github.com/wkarts/argws-ponto-manager/commit/633da88d92f8a205b3d1688c001199927d0ec71f))

## [1.16.4](https://github.com/wkarts/argws-ponto-manager/compare/v1.16.3...v1.16.4) (2026-04-05)


### Bug Fixes

* **cartao-ponto:** restaura guias e expansão da grade ao recolher sidebar ([0cf95d2](https://github.com/wkarts/argws-ponto-manager/commit/0cf95d2a63fe29e0f0395b019a3a1ed45ef82a95))

## [1.16.3](https://github.com/wkarts/argws-ponto-manager/compare/v1.16.2...v1.16.3) (2026-04-05)


### Bug Fixes

* **cartao-ponto:** evita conflito TS2451 na sidebar colapsável ([ed013e9](https://github.com/wkarts/argws-ponto-manager/commit/ed013e94fd7d88109c7ce54d35a8e1736e69ff9f))

## [1.16.2](https://github.com/wkarts/argws-ponto-manager/compare/v1.16.1...v1.16.2) (2026-04-05)


### Bug Fixes

* **jornadas:** tolera schema legado sem colunas flexíveis ([0f790d3](https://github.com/wkarts/argws-ponto-manager/commit/0f790d3779a4a79fd92694cb3a2998581fbde421))

## [1.16.1](https://github.com/wkarts/argws-ponto-manager/compare/v1.16.0...v1.16.1) (2026-04-05)


### Bug Fixes

* **build:** corrige falha no cargo fmt ajustando formatação do timecalc ([53253b8](https://github.com/wkarts/argws-ponto-manager/commit/53253b84895ce1009d5b373e91e71d32fd1e8081))
* **build:** corrige falha no cargo fmt na validação de data final de férias ([0935b2d](https://github.com/wkarts/argws-ponto-manager/commit/0935b2dab4e4ed53a6142c4dda049107818d3ac1))
* **cartao-ponto:** move recursos para sidebar lateral e considera pré-admissão na apuração ([6f1c7f6](https://github.com/wkarts/argws-ponto-manager/commit/6f1c7f67cf825063d08e39c99e0b26f02114e00d))
* **employees:** corrige erro de build ao restaurar campos de férias no save ([8a8448d](https://github.com/wkarts/argws-ponto-manager/commit/8a8448d1ade300032383f862432c1d855df7bcf5))

# [1.16.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.15.0...v1.16.0) (2026-04-04)


### Bug Fixes

* **build:** corrige falha no cargo fmt --check ajustando formatação de arquivos Rust ([607758b](https://github.com/wkarts/argws-ponto-manager/commit/607758b91cb1e005b4ed9e711c510103e35e4866))
* **clippy:** restaura variáveis de férias no save de funcionários ([dc997da](https://github.com/wkarts/argws-ponto-manager/commit/dc997daca74fb1621c6182d5f717c46996dda70e))
* **clippy:** restaura variáveis de férias no save de funcionários ([12f7f3e](https://github.com/wkarts/argws-ponto-manager/commit/12f7f3ef17076003f44c1b08a2c4a9efc2ed0525))
* **rustfmt:** ajusta formatação dos blocos de desligamento e férias em timecalc ([3a0818c](https://github.com/wkarts/argws-ponto-manager/commit/3a0818cec774983648839143e7f5b004c7c78252))


### Features

* **cartao-ponto:** adiciona sidebar lateral com guias, reforça splash global e considera férias/desligamento na apuração ([3c21907](https://github.com/wkarts/argws-ponto-manager/commit/3c219078977996500ec571d815d1085e7ce41ff7))

# [1.15.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.14.0...v1.15.0) (2026-04-04)


### Features

* **cartao-ponto:** adiciona edição inline real, navegação por teclado e fechamento operacional da grade principal ([ebdae8a](https://github.com/wkarts/argws-ponto-manager/commit/ebdae8afde2cf371d6c62533e8c179745a8e41e7))

# [1.14.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.13.0...v1.14.0) (2026-04-04)


### Bug Fixes

* **timecalc:** ajusta formatação rustfmt nas regras de jornada flexível ([f29695b](https://github.com/wkarts/argws-ponto-manager/commit/f29695b44aba1a6cab21d3d2d7d132382d9a451d))


### Features

* **ponto:** aplica complemento final na base atual com jornada flexível, smart avançado e preservação dos recursos existentes ([1bd9916](https://github.com/wkarts/argws-ponto-manager/commit/1bd99163bb1bef98074dc4e4c385dfaec9079a07))

# [1.13.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.12.0...v1.13.0) (2026-04-04)


### Features

* **cartao-ponto:** reorganiza layout operacional no estilo VB6 e melhora painel smart ([ca25ef0](https://github.com/wkarts/argws-ponto-manager/commit/ca25ef06dd6b8e879b552b9370a2b82c9b99b83d))

# [1.12.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.11.0...v1.12.0) (2026-04-04)


### Features

* **cartao-ponto:** reorganiza layout no estilo VB6 e move revisão smart para modal ([15c87b5](https://github.com/wkarts/argws-ponto-manager/commit/15c87b53b2e2f1a6e7024ea75422df2bce93b8e6))

# [1.11.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.10.0...v1.11.0) (2026-04-04)


### Bug Fixes

* **clippy:** corrige condição redundante na aplicação smart de tratamentos ([29af429](https://github.com/wkarts/argws-ponto-manager/commit/29af429187469b3f7a73fe018765f335db31d894))
* **rustfmt:** ajusta diffs finais em punches treatments e models ([45511d2](https://github.com/wkarts/argws-ponto-manager/commit/45511d29d3f8d7dd6d6e754d251cab933fa8def1))


### Features

* **cartao-ponto:** adiciona motor smart, exclusão assistida de duplicadas e reorganização operacional ([f0e268e](https://github.com/wkarts/argws-ponto-manager/commit/f0e268e2a1ff3d2b409323c785ed31c5fc1aa9b7))

# [1.10.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.9.2...v1.10.0) (2026-04-04)


### Bug Fixes

* **clippy:** corrige erros de compilação em timecalc, employees e jornadas ([b18ba55](https://github.com/wkarts/argws-ponto-manager/commit/b18ba55a9c313ad2c2043a35f56cbd7a754237ee))
* **clippy:** corrige tuple mismatch e borrows em timecalc employees e jornadas ([2b27062](https://github.com/wkarts/argws-ponto-manager/commit/2b270629cc0fe19dbbba137ef70156b0f74c37f6))
* **jornadas:** corrige delimitador não fechado em jornada_save ([f6fade8](https://github.com/wkarts/argws-ponto-manager/commit/f6fade84264d967cb0c510b49b797252ab180848))
* **rustfmt:** ajusta arquivos pendentes do cargo fmt check ([2865326](https://github.com/wkarts/argws-ponto-manager/commit/28653263920f970b015907b3cf070e352fa39ebb))


### Features

* **jornada:** adiciona perfis flexíveis, heurística de folga móvel, importação de funcionários e filtros compactos ([030cc36](https://github.com/wkarts/argws-ponto-manager/commit/030cc36070e5034d35cdc4854510f589d898c692))

## [1.9.2](https://github.com/wkarts/argws-ponto-manager/compare/v1.9.1...v1.9.2) (2026-04-03)


### Bug Fixes

* **relatorio-horas:** evitar erro ao registrar exportacao PDF ([29b3654](https://github.com/wkarts/argws-ponto-manager/commit/29b3654b03a3b38d7c6a4444f5c6b25c06c7bef9))

## [1.9.1](https://github.com/wkarts/argws-ponto-manager/compare/v1.9.0...v1.9.1) (2026-04-03)


### Bug Fixes

* **relatorios:** imprimir e exportar relatório formatado em A4 ([be04dfc](https://github.com/wkarts/argws-ponto-manager/commit/be04dfc98c63ef80d9a8d42a1ec304217d1965bf))

# [1.9.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.8.3...v1.9.0) (2026-04-02)


### Bug Fixes

* **rust:** completar ApuracaoRequest em chamadas internas ([8dc10e3](https://github.com/wkarts/argws-ponto-manager/commit/8dc10e34df99042a8318f8a7de744c8cb571ed5f))


### Features

* **relatorios:** adicionar tela de relatório consolidado de horas ([dfebbde](https://github.com/wkarts/argws-ponto-manager/commit/dfebbde796565c5c968fd203e69f11d6d1a61472))

## [1.8.3](https://github.com/wkarts/argws-ponto-manager/compare/v1.8.2...v1.8.3) (2026-04-02)


### Bug Fixes

* **employees:** normaliza CPF legado com zero à esquerda no save ([71a484d](https://github.com/wkarts/argws-ponto-manager/commit/71a484d3e95c975ff39f1d29985b61668ef5a32a))

## [1.8.2](https://github.com/wkarts/argws-ponto-manager/compare/v1.8.1...v1.8.2) (2026-04-02)


### Bug Fixes

* **ui:** corrige typecheck final de formulários, switches e textareas ([0071dd5](https://github.com/wkarts/argws-ponto-manager/commit/0071dd54a1d45f757c8dcfd50e5f3d81d109e5ae))
* **ui:** corrige typecheck nas páginas de cadastro e ajusta fmt de companies.rs ([84aaed8](https://github.com/wkarts/argws-ponto-manager/commit/84aaed8889d738b9581d61de637feb03fb1c56fd))
* **ui:** refina sincronização técnica, substitui booleanos por switch e corrige validação de documento da empresa ([16db455](https://github.com/wkarts/argws-ponto-manager/commit/16db455e55159d5810b9fcc129d00ef381fd3d95))

## [1.8.1](https://github.com/wkarts/argws-ponto-manager/compare/v1.8.0...v1.8.1) (2026-04-02)


### Bug Fixes

* **build:** ajusta formatação dos módulos fiscais e de empresas para passar no rustfmt ([cf83768](https://github.com/wkarts/argws-ponto-manager/commit/cf8376803878423d7034e09563c87bbae4b30c44))
* **build:** corrige campo atraso_minutes na montagem de DailyCalculation ([68640c0](https://github.com/wkarts/argws-ponto-manager/commit/68640c002b9cd7b2f07363afaa8a939bf4c95b55))
* **build:** corrige erros de build e adiciona consulta CNPJ/IE com fonte genérica de feriados ([933e87d](https://github.com/wkarts/argws-ponto-manager/commit/933e87d6631d065152b2b42b043d2d6aa819093c))
* **build:** corrige falhas de clippy/check e inclui dataset embarcado de feriados 2026 ([99bf2fe](https://github.com/wkarts/argws-ponto-manager/commit/99bf2feedd535bbbaf35f959971102bd7c7e4a87))
* **build:** corrige falhas de clippy/check e inclui dataset embarcado de feriados 2026 ([6c1f1d0](https://github.com/wkarts/argws-ponto-manager/commit/6c1f1d0b70f5b7e6d6563c8368e3d02f4094fc36))
* **build:** resolve erro de dead_code na struct ResolvedSchedule ([35b421c](https://github.com/wkarts/argws-ponto-manager/commit/35b421ca812d9bfb6191ecd72f558892b7365c59))
* **feriados:** organiza abrangência por empresa e departamento com compatibilidade retroativa ([d0adcc5](https://github.com/wkarts/argws-ponto-manager/commit/d0adcc50e1c6e46f00a5049103e9fe19d0da54ae))

# [1.8.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.7.0...v1.8.0) (2026-04-02)


### Bug Fixes

* **ci:** corrige falha no clippy (-D warnings) em timecalc (dead_code) ([3df421a](https://github.com/wkarts/argws-ponto-manager/commit/3df421abaa0f8111147c9d401f8bd48ca6f2f1f6))
* **feriados:** corrige typecheck da tela e ajusta formatação do backend no CI ([acbd95c](https://github.com/wkarts/argws-ponto-manager/commit/acbd95c46821a7d6566f5e6bebec07019a24bd2e))
* **feriados:** corrige typecheck da tela e ajusta formatação do backend no CI ([11e5325](https://github.com/wkarts/argws-ponto-manager/commit/11e5325c999702f6568a3ec1ebbaf5a5a601438f))


### Features

* **feriados:** integra módulo dedicado de feriados com modal e apuração contextual ([05ee565](https://github.com/wkarts/argws-ponto-manager/commit/05ee565c41997eb63fec4076dbf14fce66971385))

# [1.7.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.6.2...v1.7.0) (2026-04-01)


### Bug Fixes

* **ci:** corrige typecheck da JornadaPage e formatação de entities.rs ([579849b](https://github.com/wkarts/argws-ponto-manager/commit/579849bfdb47a2b10f8e3ae8637939c9c830ce7a))


### Features

* **ui:** aplica modal nas páginas restantes de CRUD e ajustes operacionais ([d047037](https://github.com/wkarts/argws-ponto-manager/commit/d047037d785d6fd99e2aaf5fbf126977a82b7dc6))
* **ui:** padroniza todas as páginas de CRUD com inclusão e edição em modal ([b07de38](https://github.com/wkarts/argws-ponto-manager/commit/b07de38dfbd75815026e3f06e929cfacf1c6ee58))

## [1.6.2](https://github.com/wkarts/argws-ponto-manager/compare/v1.6.1...v1.6.2) (2026-03-28)


### Bug Fixes

* **rust:** accept Path in generated report save helper ([4e20904](https://github.com/wkarts/argws-ponto-manager/commit/4e20904c6af8620cbd6961a6f7022a777a1db303))

## [1.6.1](https://github.com/wkarts/argws-ponto-manager/compare/v1.6.0...v1.6.1) (2026-03-28)


### Bug Fixes

* **cartao-ponto:** base report models on real apuracao rules ([8f17cab](https://github.com/wkarts/argws-ponto-manager/commit/8f17cab59cb198ed49c174c79c810371a05e0a04))

# [1.6.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.5.0...v1.6.0) (2026-03-28)


### Features

* **cartao-ponto:** add four folha report models for export/print ([91695f5](https://github.com/wkarts/argws-ponto-manager/commit/91695f5b83eac5a7c51ece4aeba7fa77209dd836))
* **cartao-ponto:** add four folha report models for export/print ([7aa9ff1](https://github.com/wkarts/argws-ponto-manager/commit/7aa9ff14fab8535fec30375004e7f5e9a3c8dbb7))
* **cartao-ponto:** add four folha report models for export/print ([c8c88fe](https://github.com/wkarts/argws-ponto-manager/commit/c8c88fe1022b36ef20ecaae1fc0f0bd1175515a8))

# [1.5.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.4.0...v1.5.0) (2026-03-28)


### Features

* **ui:** add in-app user and technical documentation pages ([0944dff](https://github.com/wkarts/argws-ponto-manager/commit/0944dff7d530325fc67316ee01ca7df8f7036110))

# [1.4.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.3.1...v1.4.0) (2026-03-28)


### Features

* **institucional:** adiciona logo e suporte nos relatorios e tela sobre ([d9e0d86](https://github.com/wkarts/argws-ponto-manager/commit/d9e0d86ceb0a1d28c53f9080fa601931b277d3ab))

## [1.3.1](https://github.com/wkarts/argws-ponto-manager/compare/v1.3.0...v1.3.1) (2026-03-28)


### Bug Fixes

* **cartao-ponto:** restaura menu batidas e acoes diretas na grade ([27d7d3c](https://github.com/wkarts/argws-ponto-manager/commit/27d7d3c17d0d04c4af10386520cd883121b15cbe))
* **cartao-ponto:** unifica acesso e impressao sem popup ([5fc5f6e](https://github.com/wkarts/argws-ponto-manager/commit/5fc5f6e96babde31ce031714eb725f0fa6f445fe))

# [1.3.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.2.0...v1.3.0) (2026-03-28)


### Features

* **relatorios:** exportacao com dialogo e layout profissional do cartao ([c67e7b3](https://github.com/wkarts/argws-ponto-manager/commit/c67e7b3796453f7cae14cc20a3aea4ea3ae3c410))

# [1.2.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.1.5...v1.2.0) (2026-03-28)


### Bug Fixes

* **rustfmt:** ajusta formatacao da migration de bootstrap ([fdf1ce5](https://github.com/wkarts/argws-ponto-manager/commit/fdf1ce5a45e221ff833f874b762354498df6a9f6))


### Features

* **cartao-ponto:** centraliza operacoes e bloqueia reseed em base ativa ([9c821df](https://github.com/wkarts/argws-ponto-manager/commit/9c821dfd6536c53e10788ba950a6397f303e5e57))

## [1.1.5](https://github.com/wkarts/argws-ponto-manager/compare/v1.1.4...v1.1.5) (2026-03-28)


### Bug Fixes

* **apuracao:** converte funcionarioId para número no payload ([2bd50cb](https://github.com/wkarts/argws-ponto-manager/commit/2bd50cb59cb010b370be44ec2c457503fb4fb6c4))

## [1.1.4](https://github.com/wkarts/argws-ponto-manager/compare/v1.1.3...v1.1.4) (2026-03-27)


### Bug Fixes

* **batidas:** envia funcionarioId como número no filtro da listagem ([5f93cca](https://github.com/wkarts/argws-ponto-manager/commit/5f93cca4dacc818082397d2aa7aadf536cd6fc7d))

## [1.1.3](https://github.com/wkarts/argws-ponto-manager/compare/v1.1.2...v1.1.3) (2026-03-27)


### Bug Fixes

* **batidas:** evita crash da rota ao tratar falhas de carregamento ([961f5e8](https://github.com/wkarts/argws-ponto-manager/commit/961f5e8fb2322317a7da1adef9a788c9b72c8df8))

## [1.1.2](https://github.com/wkarts/argws-ponto-manager/compare/v1.1.1...v1.1.2) (2026-03-27)


### Bug Fixes

* **release:** define repositoryUrl padrão no semantic-release ([92a464c](https://github.com/wkarts/argws-ponto-manager/commit/92a464c070cb1f69874bbd1032429a4645db7580))

## [1.1.1](https://github.com/wkarts/argws-ponto-manager/compare/v1.1.0...v1.1.1) (2026-03-27)


### Bug Fixes

* **app:** corrige sessão, tela branca, cadastros administrativos e reforça logs e segurança ([10248de](https://github.com/wkarts/argws-ponto-manager/commit/10248de0a55b1005a0a53845ea0c99db2a66ec5d))
* **ci:** corrige TS2339 em App/main e ajusta cargo fmt nos módulos de sessão e licenciamento ([1749d04](https://github.com/wkarts/argws-ponto-manager/commit/1749d04cb0c5a49e75e8de67f65bb371f844712b))
* **clippy:** corrige runtime de licenciamento e assinatura de logging no Ponto Manager ([27f6c44](https://github.com/wkarts/argws-ponto-manager/commit/27f6c445fd50b092999f4c954706bb95b9767ead))
* **clippy:** corrige unused variable em licensing e reduz argumentos de write_app_log ([1fd91f6](https://github.com/wkarts/argws-ponto-manager/commit/1fd91f612732c4682effe6a4cff2d2a2d27b74b9))

# [1.1.0](https://github.com/wkarts/argws-ponto-manager/compare/v1.0.0...v1.1.0) (2026-03-27)


### Bug Fixes

* **treatments:** corrige montagem de ApuracaoRequest no fechamento mensal ([68c22a3](https://github.com/wkarts/argws-ponto-manager/commit/68c22a30ebe661a0faf0d4ba61c24ad36130002a))


### Features

* **licensing:** integra generic-license-tauri com página dedicada e licença teste de 45 dias ([231044d](https://github.com/wkarts/argws-ponto-manager/commit/231044da4b1eb17390a0cb4b021dba6caaa2d2c3))
* **ponto:** adiciona edição em lote de batidas e suporte a build windows x86 i686 ([5485248](https://github.com/wkarts/argws-ponto-manager/commit/5485248063bf63a6d1f9341411be2e928cc0e08c))

# 1.0.0 (2026-03-26)


### Bug Fixes

* **ci:** corrige erros de typecheck e ajustes de cargo fmt na aplicação Ponto ([2695f40](https://github.com/wkarts/argws-ponto-manager/commit/2695f408df170ade101202e8713ba562ae236675))
* **clippy:** corrige falhas de lint em db, auth, timecalc, afd, entities, jornadas e lib ([f1a5f67](https://github.com/wkarts/argws-ponto-manager/commit/f1a5f679dd28120664ba07aa0ddb1b75c38e47d5))


### Features

* **branding:** adiciona kit visual da aplicação Ponto e integra assets ao Tauri ([5681160](https://github.com/wkarts/argws-ponto-manager/commit/5681160ae7262a927da82b3cc667e7df7991c213))
* **ponto manager:** New System Ponto Manager ([7793117](https://github.com/wkarts/argws-ponto-manager/commit/7793117651e86a6bdbb496b7aaee7471f6eb5499))

# Changelog

Todas as mudanças relevantes deste projeto serão registradas aqui.
