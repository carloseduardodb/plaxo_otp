# 🔐 Plaxo OTP

Uma aplicação desktop moderna e segura para geração de códigos OTP (One-Time Password) com criptografia de nível bancário.

## 📸 Screenshots

<!-- ADICIONE AQUI A IMAGEM DA APLICAÇÃO -->

![Plaxo OTP Interface](./docs/screenshot-01.png)

<br/>

![Plaxo OTP Interface](./docs/screenshot-02.png)

<br/>

![Plaxo OTP Interface](./docs/screenshot-03.png)

## ✨ Características

### 🛡️ **Segurança Máxima**

- **Criptografia AES-256-GCM** - Padrão militar com autenticação integrada
- **Derivação de chave SHA-256** - Com salt único para máxima proteção
- **Nonce aleatório** - Cada criptografia é única
- **Dados sempre criptografados** - Nunca armazenados em texto plano
- **Chave apenas na sessão** - Senha mestre não persiste no disco

### 🎨 **Interface Moderna**

- **Design System Plaxo** - Interface profissional e consistente
- **Ícones das plataformas** - Reconhecimento visual automático
- **Tema escuro** - Reduz fadiga visual
- **Responsivo** - Adapta-se ao tamanho da janela
- **Animações suaves** - Experiência fluida

### ⚡ **Funcionalidades**

- **Geração OTP automática** - Códigos TOTP de 6 dígitos
- **Atualização em tempo real** - Renovação a cada 30 segundos
- **Barra de progresso visual** - Indica tempo restante
- **Cópia rápida** - Um clique para copiar código
- **Importação 2FAS** - Migre facilmente do 2FAS
- **Pesquisa inteligente** - Encontre apps rapidamente
- **System tray** - Acesso rápido pela bandeja do sistema

## 🚀 Download

### Releases Automáticos

Baixe a versão mais recente na [página de releases](https://github.com/carloseduardodb/plaxo_otp/releases/latest):

- **Windows**: `Plaxo-OTP_x.x.x_x64_en-US.msi`
- **macOS**: `Plaxo-OTP_x.x.x_x64.dmg`
- **Linux**: `plaxo-otp_x.x.x_amd64.deb` ou `plaxo-otp_x.x.x_amd64.AppImage`

## 🛠️ Desenvolvimento

### Pré-requisitos

- **Node.js** 18+
- **Rust** 1.70+
- **Tauri CLI**: `npm install -g @tauri-apps/cli`

### Dependências do Sistema

#### Ubuntu/Debian

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

#### Fedora

```bash
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
```

#### macOS

```bash
# Xcode Command Line Tools são suficientes
xcode-select --install
```

### Instalação

```bash
# Clone o repositório
git clone https://github.com/carloseduardodb/plaxo_otp.git
cd plaxo-otp

# Instale dependências
npm install

# Execute em modo desenvolvimento
npm run tauri dev

# Build para produção
npm run tauri build
```

## 📱 Como Usar

### Primeira Configuração

1. **Abra a aplicação** - Ela aparecerá na bandeja do sistema
2. **Clique no ícone** da bandeja para abrir
3. **Crie sua senha mestre** - Esta será usada para criptografar seus dados
4. **Adicione seus primeiros apps** - Use o botão "+" ou importe do 2FAS

### Adicionando Aplicativos

1. **Clique no botão "+"** no topo da aplicação
2. **Digite o nome** do aplicativo (ex: "Google", "GitHub")
3. **Cole a chave secreta** do 2FA (código QR em texto)
4. **Clique em "Adicionar"**

### Importando do 2FAS

1. **No app 2FAS**: Configurações → Backup → Exportar
2. **No Plaxo OTP**: Clique em "Importar do 2FAS"
3. **Selecione o arquivo** .2fas exportado
4. **Aguarde a importação** - Todos os apps serão adicionados

### Usando Códigos OTP

- **Códigos são gerados automaticamente** a cada 30 segundos
- **Clique em "Copiar"** para copiar o código atual
- **Barra de progresso** mostra tempo restante
- **Pesquise** pelo nome do aplicativo

## 🔒 Segurança

### Criptografia

- **AES-256-GCM**: Algoritmo de criptografia militar
- **SHA-256**: Hash criptográfico para derivação de chave
- **Nonce aleatório**: Cada criptografia é única
- **Salt único**: Proteção contra rainbow tables

### Armazenamento

- **Arquivo criptografado**: `~/.plaxo-otp/apps.enc`
- **Chave na memória**: Nunca salva no disco
- **Permissões de usuário**: Apenas você pode acessar

### Comparação de Segurança

| Aplicativo           | Criptografia Local | Nível de Segurança |
| -------------------- | ------------------ | ------------------ |
| Google Authenticator | ❌                 | Básico             |
| Authy                | ✅ (Nuvem)         | Médio              |
| 1Password            | ✅                 | Alto               |
| **Plaxo OTP**        | ✅                 | **Máximo**         |

## 🏗️ Arquitetura

### Frontend

- **React 18** - Interface moderna e reativa
- **TypeScript** - Tipagem estática para maior confiabilidade
- **Tailwind CSS** - Design system consistente
- **Lucide React** - Ícones profissionais

### Backend

- **Rust** - Performance e segurança máximas
- **Tauri** - Framework para aplicações desktop
- **TOTP-RS** - Geração de códigos OTP
- **AES-GCM** - Criptografia de dados

### Dependências Principais

```toml
[dependencies]
tauri = "1.5"
serde = "1.0"
totp-rs = "5.0"
aes-gcm = "0.10"
sha2 = "0.10"
uuid = "1.0"
```

## Roadmap
- Sincronização em nuvem.
- Aceite colar imagem QRCode ao adicionar novo aplicativo.
- Monitoramento do salvamento em nuvem.
- Novo app mobile.
- Compartilhamento de codigo de apps entre dispositivos.

## 🤝 Contribuindo

1. **Fork** o projeto
2. **Crie uma branch** para sua feature (`git checkout -b feature/nova-feature`)
3. **Commit** suas mudanças (`git commit -am 'Adiciona nova feature'`)
4. **Push** para a branch (`git push origin feature/nova-feature`)
5. **Abra um Pull Request**

## 📄 Licença

Este projeto está licenciado sob a licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## 🆘 Suporte

- **Issues**: [GitHub Issues](https://github.com/carloseduardodb/plaxo_otp/issues)
- **Discussões**: [GitHub Discussions](https://github.com/carloseduardodb/plaxo_otp/discussions)
- **Email**: suporte@plaxo.com

## 🙏 Agradecimentos

- **Tauri Team** - Framework incrível para apps desktop
- **Rust Community** - Linguagem segura e performática
- **React Team** - Interface moderna e reativa
- **2FAS** - Inspiração para funcionalidades

---

<div align="center">
  <strong>Feito com ❤️ pela equipe Plaxo</strong>
</div>
