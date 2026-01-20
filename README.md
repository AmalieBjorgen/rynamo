# Rynamo

A terminal user interface (TUI) for exploring Microsoft Dataverse and Dynamics 365 metadata.

## Features

- **Entity Browser**: Browse all entities (tables) in your Dataverse environment
- **Attribute Explorer**: View columns, types, and requirements for each entity
- **Relationship Viewer**: Explore 1:N, N:1, and N:N relationships
- **Solution Browser**: List and explore solutions in your environment
- **Solution Layer Explorer**: Understand component customization history and managed/unmanaged layers
- **FetchXML Console**: Execute direct FetchXML queries against your environment
- **Environment Discovery**: Automatically discover Dataverse environments via Azure CLI
- **User & Security Explorer**: View users, teams, and security role assignments (direct and inherited)
- **Search/Filter**: Quickly filter entities, attributes, and solutions by name
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

### Global

| Key | Action |
|-----|--------|
| `1` | Go to Entities view |
| `2` | Go to Solutions view |
| `3` | Go to Users view |
| `4` | Go to Global OptionSets view |
| `G` | Global metadata search |
| `E` | Environment switcher |
| `f` / `F` | Open FetchXML Console |
| `/` | Open search/filter popup |
| `q` | Quit or Go Back |
| `Esc` | Go back |

### Navigation (Default)

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate up/down |
| ←/→ | Switch tabs (in detail views) |
| Tab | Next tab |
| Enter | Select/Open details |

### Specialized

| Key | Action |
|-----|--------|
| `L` | View Solution Layers for selected component |
| `D` | Discover environments (in Environment view) |

### Vim Mode (--vim)

| Key | Action |
|-----|--------|
| `j`/`k` | Navigate up/down |
| `h`/`l` | Switch tabs |

## Requirements

- Rust 1.75+
- Azure CLI installed and logged in
- Access to a Dataverse environment

## License

MIT
