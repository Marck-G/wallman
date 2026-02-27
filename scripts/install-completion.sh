#!/bin/bash

# Wallman completion installation script
# This script helps install shell completions for wallman

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WALLMAN_BIN="${WALLMAN_BIN:-wallman}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to detect shell
detect_shell() {
    if [ -n "$ZSH_VERSION" ]; then
        echo "zsh"
    elif [ -n "$BASH_VERSION" ]; then
        echo "bash"
    elif [ -n "$FISH_VERSION" ]; then
        echo "fish"
    elif [ -n "$POSH_VERSION" ]; then
        echo "powershell"
    else
        # Try to detect from SHELL environment variable
        case "$SHELL" in
            */zsh) echo "zsh" ;;
            */bash) echo "bash" ;;
            */fish) echo "fish" ;;
            */pwsh|*/powershell) echo "powershell" ;;
            *) echo "bash" ;; # Default to bash
        esac
    fi
}

# Function to install completion using wallman's built-in command
install_with_wallman() {
    local shell="$1"
    local force="$2"
    
    print_info "Installing completion for $shell using wallman..."
    
    if [ "$force" = "true" ]; then
        if $WALLMAN_BIN completion install --force 2>&1; then
            print_success "Completion installed successfully"
            return 0
        else
            print_error "Failed to install completion"
            return 1
        fi
    else
        if $WALLMAN_BIN completion install 2>&1; then
            print_success "Completion installed successfully"
            return 0
        else
            print_error "Failed to install completion"
            return 1
        fi
    fi
}

# Function to manually install completion
install_manually() {
    local shell="$1"
    local force="$2"
    
    print_info "Generating completion script for $shell..."
    
    case "$shell" in
        "bash")
            completion_file="${HOME}/.local/share/bash-completion/completions/wallman"
            if [ ! -d "$(dirname "$completion_file")" ]; then
                mkdir -p "$(dirname "$completion_file")"
            fi
            ;;
        "zsh")
            completion_file="${HOME}/.zsh/completions/_wallman"
            if [ ! -d "$(dirname "$completion_file")" ]; then
                mkdir -p "$(dirname "$completion_file")"
            fi
            ;;
        "fish")
            completion_file="${HOME}/.config/fish/completions/wallman.fish"
            if [ ! -d "$(dirname "$completion_file")" ]; then
                mkdir -p "$(dirname "$completion_file")"
            fi
            ;;
        *)
            print_error "Manual installation not supported for shell: $shell"
            return 1
            ;;
    esac
    
    if [ -f "$completion_file" ] && [ "$force" != "true" ]; then
        print_warning "Completion file already exists: $completion_file"
        print_warning "Use --force to overwrite"
        return 1
    fi
    
    if $WALLMAN_BIN completion generate "$shell" > "$completion_file" 2>/dev/null; then
        print_success "Completion script generated: $completion_file"
        
        # Add shell-specific instructions
        case "$shell" in
            "bash")
                print_info "To enable completion, add the following to your ~/.bashrc:"
                echo "  source $completion_file"
                ;;
            "zsh")
                print_info "To enable completion, add the following to your ~/.zshrc:"
                echo "  fpath+=($completion_file)"
                echo "  autoload -U compinit && compinit"
                ;;
            "fish")
                print_info "Completion should be automatically loaded for fish"
                ;;
        esac
        
        return 0
    else
        print_error "Failed to generate completion script"
        return 1
    fi
}

# Function to show installation instructions
show_instructions() {
    local shell="$1"
    
    print_info "Manual installation instructions for $shell:"
    echo ""
    
    case "$shell" in
        "bash")
            echo "1. Generate the completion script:"
            echo "   $WALLMAN_BIN completion generate bash > ~/.local/share/bash-completion/completions/wallman"
            echo ""
            echo "2. Add to your ~/.bashrc:"
            echo "   source ~/.local/share/bash-completion/completions/wallman"
            echo ""
            echo "3. Reload your shell:"
            echo "   source ~/.bashrc"
            ;;
        "zsh")
            echo "1. Generate the completion script:"
            echo "   $WALLMAN_BIN completion generate zsh > ~/.zsh/completions/_wallman"
            echo ""
            echo "2. Add to your ~/.zshrc:"
            echo "   fpath+=($HOME/.zsh/completions)"
            echo "   autoload -U compinit && compinit"
            echo ""
            echo "3. Reload your shell:"
            echo "   source ~/.zshrc"
            ;;
        "fish")
            echo "1. Generate the completion script:"
            echo "   $WALLMAN_BIN completion generate fish > ~/.config/fish/completions/wallman.fish"
            echo ""
            echo "2. Reload your shell or run:"
            echo "   source ~/.config/fish/completions/wallman.fish"
            ;;
        "powershell")
            echo "1. Generate the completion script:"
            echo "   $WALLMAN_BIN completion generate powershell > \$PROFILE"
            echo ""
            echo "2. Reload your PowerShell session"
            ;;
        *)
            echo "Shell $shell is not supported for manual installation"
            ;;
    esac
}

# Function to uninstall completion
uninstall_completion() {
    local shell="$1"
    
    print_info "Uninstalling completion for $shell..."
    
    if $WALLMAN_BIN completion uninstall 2>&1; then
        print_success "Completion uninstalled successfully"
        return 0
    else
        print_error "Failed to uninstall completion"
        return 1
    fi
}

# Function to show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -i, --install     Install completion for current shell"
    echo "  -u, --uninstall   Uninstall completion for current shell"
    echo "  -m, --manual      Show manual installation instructions"
    echo "  -f, --force       Force overwrite existing completion"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 -i                    # Install completion for current shell"
    echo "  $0 -i -f                 # Force install completion"
    echo "  $0 -u                    # Uninstall completion"
    echo "  $0 -m                    # Show manual installation instructions"
    echo ""
    echo "Environment variables:"
    echo "  WALLMAN_BIN              Path to wallman binary (default: wallman)"
}

# Parse command line arguments
INSTALL=false
UNINSTALL=false
MANUAL=false
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--install)
            INSTALL=true
            shift
            ;;
        -u|--uninstall)
            UNINSTALL=true
            shift
            ;;
        -m|--manual)
            MANUAL=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check if wallman binary exists
if ! command -v "$WALLMAN_BIN" &> /dev/null; then
    print_error "wallman binary not found: $WALLMAN_BIN"
    print_info "Please ensure wallman is installed and available in your PATH"
    print_info "Or set the WALLMAN_BIN environment variable to the correct path"
    exit 1
fi

# Detect current shell
CURRENT_SHELL=$(detect_shell)
print_info "Detected shell: $CURRENT_SHELL"

# Perform requested action
if [ "$INSTALL" = "true" ]; then
    if ! install_with_wallman "$CURRENT_SHELL" "$FORCE"; then
        print_warning "Falling back to manual installation..."
        install_manually "$CURRENT_SHELL" "$FORCE"
    fi
elif [ "$UNINSTALL" = "true" ]; then
    uninstall_completion "$CURRENT_SHELL"
elif [ "$MANUAL" = "true" ]; then
    show_instructions "$CURRENT_SHELL"
else
    print_info "No action specified. Use -h for help."
    show_instructions "$CURRENT_SHELL"
fi