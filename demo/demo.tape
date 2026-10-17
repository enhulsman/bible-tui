# bible-tui demo recording.
#
# Render from the repository root after `cargo build --release -p bible-native`:
#
#   command vhs demo/demo.tape
#
# The session runs against a throwaway HOME (mktemp, or $DEMO_HOME if set, which
# should be an empty directory), so your real state and bookmarks are never read
# or touched.
#
# vhs's own GIF is ~8 MB, so it is re-encoded in place afterwards: 8 fps with
# duplicate frames dropped, and a 48-colour palette seeded with the theme's
# accent colours (otherwise the small cyan/yellow areas wash out to grey):
#
#   palettegen:  [gif]mpdecimate,fps=8 + [swatch] -> concat -> palettegen=max_colors=48:stats_mode=full
#   paletteuse:  mpdecimate,fps=8 -> paletteuse=dither=none:diff_mode=rectangle (-fps_mode vfr)
#
# The poster is the command-palette frame:
#   ffmpeg -ss 14.6 -i demo/demo.gif -frames:v 1 demo/poster.png   (on vhs's raw GIF)

Output demo/demo.gif
Output demo/demo.webm
Output demo/demo.mp4

Set Shell "bash"
Set FontFamily "JetBrains Mono"
Set FontSize 18
Set Width 1200
Set Height 700
Set Padding 24
Set WindowBar Colorful
Set BorderRadius 8
Set Framerate 30
Set TypingSpeed 75ms
Set CursorBlink false
Set Theme {"name":"hulsman","background":"#141018","foreground":"#f0ead6","cursor":"#d4a030","selection":"#2a2636","black":"#1e1b28","red":"#e06c75","green":"#2dd4a8","yellow":"#d4a030","blue":"#7aa2f7","magenta":"#c678dd","cyan":"#56b6c2","white":"#f0ead6","brightBlack":"#9a9488","brightRed":"#ff7b86","brightGreen":"#5ee6c2","brightYellow":"#e2b445","brightBlue":"#9bb8ff","brightMagenta":"#d79be8","brightCyan":"#7fd0db","brightWhite":"#ffffff"}

# --- Setup (not recorded): throwaway HOME, open at John 1 in the KJV --------
Hide
Type `export PATH="$PWD/target/release:/usr/bin:/bin" HOME="${DEMO_HOME:-$(mktemp -d)}"` Enter
Type `export XDG_DATA_HOME="$HOME/.local/share" XDG_CONFIG_HOME="$HOME/.config"` Enter
Type `mkdir -p "$XDG_DATA_HOME/bible-tui" && printf 'last_book = 42\nlast_chapter = 1\nlast_translation = "KJV"\n' > "$XDG_DATA_HOME/bible-tui/state.toml"` Enter
Type `cd "$XDG_DATA_HOME/bible-tui" && PS1='$ ' && clear` Enter
Sleep 500ms
Show

# --- Launch -----------------------------------------------------------------
Sleep 300ms
Type "bible"
Sleep 300ms
Enter
Sleep 1.4s

# --- Scroll with j / k ------------------------------------------------------
Type@160ms "jjjjj"
Sleep 400ms
Type@160ms "kk"
Sleep 600ms

# --- Full-text search, narrowing as you type --------------------------------
Type "/"
Sleep 400ms
Type@100ms "light"
Sleep 1.1s
Type@45ms " world"
Sleep 900ms
# Result 6 is John 8:12 after duplicate verses are removed.
Down@300ms 5
Sleep 600ms
Enter
Sleep 1.2s

# --- Scroll within John 8 ---------------------------------------------------
Type@15ms "jjjjjjjjjjjjjjjjjjj"
Sleep 500ms

# --- Switch translation through the command palette -------------------------
Type ":"
Sleep 600ms
Type "t "
Sleep 1s
Type "s"
Sleep 400ms
Tab
Sleep 500ms
Enter
Sleep 2s
