# Rynamo

A terminal user interface (TUI) for exploring Microsoft Dataverse and Dynamics 365 metadata.

## Features

- **Entity Browser**: Browse all entities (tables) in your Dataverse environment
- **Attribute Explorer**: View columns, types, and requirements for each entity
- **Relationship Viewer**: Explore 1:N, N:1, and N:N relationships
- **Solution Browser**: List and explore solutions in your environment
- **Search/Filter**: Quickly filter entities and attributes by name
- **Azure CLI Authentication**: Uses your existing Azure CLI credentials

## Installation

```bash
cargo install --path .
```

## Usage

First, ensure you're logged in with Azure CLI:

```bash
az login
```

Then run Rynamo with your Dataverse environment URL:

```bash
rynamo --env https://yourorg.crm.dynamics.com
```

### Command Line Options

| Option | Description |
|--------|-------------|
| `--env` | Dataverse environment URL (required) |
| `--vim` | Enable vim-style keybindings (j/k navigation) |

You can also set the environment URL via the `DATAVERSE_URL` environment variable.

## Keybindings

### Default (Arrow Keys)

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate up/down |
| ←/→ | Switch tabs |
| Enter | Select/Open details |
| Esc | Go back |
| / | Open search |
| q | Quit (or go back from details) |
| 1 | Go to Entities view |
| 2 | Go to Solutions view |
| Tab | Next tab |

### Vim Mode (--vim)

| Key | Action |
|-----|--------|
| j/k | Navigate up/down |
| h/l | Switch tabs |

## Requirements

- Rust 1.75+
- Azure CLI installed and logged in
- Access to a Dataverse environment

## License

MIT
