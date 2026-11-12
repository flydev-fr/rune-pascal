# VCL hosting

TRunePanel is a TCustomControl with a direct GPU surface. VCL owns the form, HWND
and message loop. Set OnBuildView before Parent, rebuild when application state
changes, and process OnRuneEvent. The separate Rune Markdown application is a complete usage example.

Define UI in Pascal units without DFM files. Keep the DLL loaded while hosts
exist; destroy the GPU host before the HWND. Never free a control inside a Rust
callback. Static scenes stop rendering; active effects request frames through
OnIdle and GPU presentation, without permanent repaint polling or CPU pixel copies.
Each host still owns its own GPU device, so avoid one surface per tiny widget.

Ctrl+C/Ctrl+Insert are decoded as Rune commands. RUNE_UI_COPY is converted from
UTF-8 and written as Windows Unicode text. Override WriteClipboardText for another
clipboard backend. The regression suite intercepts this method to preserve the
user's clipboard while checking the full input/event path.

The control is reusable in a form; a complete set of independent VCL widgets or
an installed IDE palette package is not included. Full embedded IME, paste and
accessibility integration remain incomplete.

RuneVclTests covers host lifetime, resize, recreation, threads and copying.
RuneVclDemo retains bench/idle modes. encode_ms measures CPU compositor encoding,
not GPU execution time. Keep windows visible and avoid compiling during measurement.
