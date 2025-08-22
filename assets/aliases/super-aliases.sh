# NexusShell Super Aliases - Ultimate Command Shortcuts
# Ultra-short aliases for maximum productivity

# === NAVIGATION (1-2 characters) ===
alias .='pwd'                      # Current directory
alias ..='cd ..'                   # Up one level  
alias ...='cd ../..'               # Up two levels
alias ....='cd ../../..'           # Up three levels
alias ~='cd ~'                     # Go home
alias -='cd -'                     # Previous directory
alias /='cd /'                     # Root directory

# === FILE OPERATIONS (1-2 characters) ===
alias l='ls'                       # Basic list
alias ll='ls -la'                  # Detailed list
alias la='ls -la'                  # All files
alias lt='ls -lt'                  # Sort by time
alias lh='ls -lh'                  # Human readable
alias ls='ls --color=auto'         # Colorized ls

# === TEXT VIEWING (1 character) ===
alias c='cat'                      # Display file
alias h='head'                     # First lines
alias t='tail'                     # Last lines
alias v='vim'                      # Quick vim
alias n='nano'                     # Quick nano

# === SEARCH & FIND (1-2 characters) ===
alias g='grep --color=auto'        # Colorized grep
alias gi='grep -i --color=auto'    # Case-insensitive
alias gr='grep -r --color=auto'    # Recursive grep
alias f='find . -name'             # Quick find
alias w='which'                    # Find command

# === SYSTEM SHORTCUTS (1-2 characters) ===
alias x='exit'                     # Quick exit
alias q='exit'                     # Another quick exit
alias cls='clear'                  # Clear screen (Windows style)
alias cl='clear'                   # Even shorter clear
alias e='env'                      # Environment
alias p='ps aux'                   # Process list
alias k='kill'                     # Kill process
alias ka='killall'                 # Kill all processes

# === FILE OPERATIONS (Safety first) ===
alias cp='cp -i'                   # Safe copy
alias mv='mv -i'                   # Safe move  
alias rm='rm -i'                   # Safe remove
alias mkdir='mkdir -p'             # Create path
alias rmdir='rmdir'                # Remove directory

# === DEVELOPMENT SHORTCUTS ===
alias py='python3'                 # Python
alias py2='python2'                # Python 2
alias node='node'                  # Node.js
alias npm='npm'                    # NPM
alias pip='pip3'                   # Pip
alias git='git'                    # Git
alias cargo='cargo'                # Rust Cargo
alias code='code'                  # VS Code
alias vim='vim'                    # Vim
alias emacs='emacs'                # Emacs

# === GIT SUPER SHORTCUTS ===
alias gs='git status'              # Git status
alias ga='git add'                 # Git add
alias gc='git commit'              # Git commit
alias gp='git push'                # Git push
alias gl='git pull'                # Git pull
alias gd='git diff'                # Git diff
alias gb='git branch'              # Git branch
alias gco='git checkout'           # Git checkout
alias gm='git merge'               # Git merge
alias gr='git reset'               # Git reset

# === NETWORK UTILITIES ===
alias ping='ping -c 4'             # Ping 4 times
alias wget='wget -c'               # Resume downloads
alias curl='curl -L'               # Follow redirects
alias ssh='ssh'                    # SSH
alias scp='scp'                    # Secure copy

# === ARCHIVE OPERATIONS ===
alias tar='tar -xvf'               # Extract tar
alias zip='zip -r'                 # Recursive zip
alias unzip='unzip'                # Extract zip
alias gzip='gzip'                  # Compress
alias gunzip='gunzip'              # Decompress

# === SYSTEM INFO ===
alias df='df -h'                   # Disk usage (human)
alias du='du -h'                   # Directory usage
alias free='free -h'               # Memory (human)
alias top='top'                    # Process monitor
alias htop='htop'                  # Better top
alias ps='ps aux'                  # Process list

# === HISTORY & ALIASES ===
alias hist='history'               # Command history
alias aliases='alias'              # Show aliases
alias reload='source ~/.bashrc'    # Reload config

# === ADVANCED SHORTCUTS ===
alias tree='find . -type d | sed -e "s/[^-][^\/]*\//  |/g" -e "s/|\([^ ]\)/|-\1/"'
alias path='echo $PATH | tr ":" "\n"'                    # Show PATH nicely
alias ports='netstat -tuln'                              # Show open ports
alias ips='ip addr show | grep inet'                     # Show IP addresses

# === DATE & TIME ===
alias now='date'                   # Current date/time
alias epoch='date +%s'             # Unix timestamp
alias iso='date -u +"%Y-%m-%dT%H:%M:%SZ"'               # ISO format

# === PRODUCTIVITY ALIASES ===
alias ..l='cd .. && ls'            # Up and list
alias ..ll='cd .. && ll'           # Up and detailed list
alias mcd='mkdir -p $1 && cd $1'   # Make and enter directory
alias backup='cp $1{,.bak}'        # Quick backup

# === COLORIZED COMMANDS ===
alias ls='ls --color=auto'
alias dir='dir --color=auto'
alias vdir='vdir --color=auto'
alias grep='grep --color=auto'
alias fgrep='fgrep --color=auto'
alias egrep='egrep --color=auto'
alias diff='diff --color=auto'

# === SAFETY ALIASES ===
alias rm='rm -i'                   # Confirm before delete
alias cp='cp -i'                   # Confirm before overwrite
alias mv='mv -i'                   # Confirm before overwrite
alias ln='ln -i'                   # Confirm before link

# === DOCKER SHORTCUTS (if Docker is installed) ===
alias d='docker'                   # Docker
alias dc='docker-compose'          # Docker Compose
alias dps='docker ps'              # List containers
alias di='docker images'           # List images

# === KUBERNETES SHORTCUTS (if kubectl is installed) ===
alias k='kubectl'                  # Kubectl
alias kp='kubectl get pods'        # Get pods
alias ks='kubectl get services'    # Get services
alias kd='kubectl describe'        # Describe resource

# === CUSTOM NXSH COMMANDS ===
alias theme='ui-design'            # Theme management
alias smart='smart_alias'          # Smart alias management
alias nxsh-update='git pull && cargo build --release'  # Update NxSh
