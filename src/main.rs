use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;

const MERMAID_JS: &str = include_str!("../mermaid.min.js");

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn build_html(diagram: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Mermaid Diagram</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html, body {{
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #1e1e2e;
  }}
  .viewport {{
    width: 100%;
    height: 100%;
    overflow: hidden;
    cursor: grab;
  }}
  .viewport:active {{ cursor: grabbing; }}
  .canvas {{
    display: inline-block;
    transform-origin: 0 0;
    will-change: transform;
  }}
  .container {{
    background: #fff;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 4px 24px rgba(0,0,0,0.3);
  }}
  .container svg {{
    display: block;
  }}
  .controls {{
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    gap: 0.5rem;
    z-index: 100;
  }}
  .controls button {{
    background: #333;
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    font-size: 1rem;
    cursor: pointer;
  }}
  .controls button:hover {{ background: #555; }}
</style>
</head>
<body>
<div class="viewport" id="viewport">
<div class="canvas" id="canvas">
<div class="container">
<pre class="mermaid">{}</pre>
</div>
</div>
</div>
<div class="controls">
<button onclick="zoomIn()">+</button>
<button onclick="zoomOut()">-</button>
<button onclick="resetZoom()">Fit</button>
</div>
<script>{}</script>
<script>
mermaid.initialize({{
  startOnLoad: true,
  theme: 'default',
  securityLevel: 'loose',
  flowchart: {{
    nodeSpacing: 100,
    rankSpacing: 120,
    padding: 24,
    htmlLabels: true,
    useMaxWidth: false,
    diagramPadding: 20
  }},
  themeCSS: `
    .node rect, .node circle, .node polygon {{ stroke-width: 3px; }}
    .node .label {{ font-size: 28px; font-weight: 600; }}
    .edgeLabel {{ font-size: 24px; background: #fff; }}
    .cluster rect {{ stroke-width: 3px; }}
    .cluster .label {{ font-size: 32px; font-weight: 700; }}
    .edgePath .path {{ stroke-width: 3px; }}
  `
}});

let svg, scale = 1, fitScale = 1, tx = 0, ty = 0;
const viewport = document.getElementById('viewport');
const canvas = document.getElementById('canvas');

function applyTransform() {{
  canvas.style.transform = `translate(${{tx}}px, ${{ty}}px) scale(${{scale}})`;
}}

function zoomAt(mx, my, factor) {{
  if (!svg) return;
  const newScale = Math.max(0.1, Math.min(10, scale * factor));
  const k = newScale / scale;
  tx = mx - (mx - tx) * k;
  ty = my - (my - ty) * k;
  scale = newScale;
  applyTransform();
}}

function zoomIn() {{ zoomAt(viewport.clientWidth / 2, viewport.clientHeight / 2, 1.3); }}
function zoomOut() {{ zoomAt(viewport.clientWidth / 2, viewport.clientHeight / 2, 1 / 1.3); }}
function resetZoom() {{
  if (!svg) return;
  const cw = canvas.offsetWidth;
  const ch = canvas.offsetHeight;
  fitScale = Math.min(viewport.clientWidth / cw, viewport.clientHeight / ch) * 0.95;
  scale = fitScale;
  tx = (viewport.clientWidth - cw * scale) / 2;
  ty = (viewport.clientHeight - ch * scale) / 2;
  applyTransform();
}}

let isPanning = false, startX, startY, startTx, startTy;
viewport.addEventListener('mousedown', e => {{
  isPanning = true;
  startX = e.clientX; startY = e.clientY;
  startTx = tx; startTy = ty;
}});
viewport.addEventListener('mouseleave', () => isPanning = false);
viewport.addEventListener('mouseup', () => isPanning = false);
viewport.addEventListener('mousemove', e => {{
  if (!isPanning) return;
  e.preventDefault();
  tx = startTx + (e.clientX - startX);
  ty = startTy + (e.clientY - startY);
  applyTransform();
}});

viewport.addEventListener('wheel', e => {{
  e.preventDefault();
  const rect = viewport.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;
  zoomAt(mx, my, e.deltaY < 0 ? 1.3 : 1 / 1.3);
}}, {{ passive: false }});

setTimeout(() => {{
  svg = document.querySelector('.container svg');
  if (!svg) return;
  resetZoom();
}}, 1000);
</script>
</body>
</html>"#,
        html_escape(diagram),
        MERMAID_JS,
    )
}

fn get_temp_dir(override_dir: Option<PathBuf>) -> io::Result<PathBuf> {
    let base = override_dir.unwrap_or_else(env::temp_dir);
    let dir = base.join("mmd-viewer");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn parse_args() -> (Option<PathBuf>, String, bool) {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return (None, String::new(), false);
    }

    if args[0] == "--help" || args[0] == "-h" {
        return (None, String::new(), true);
    }

    if args[0] == "--temp-dir" {
        if args.len() < 2 {
            eprintln!("Error: --temp-dir requires a path argument");
            std::process::exit(1);
        }
        let temp_dir = PathBuf::from(&args[1]);
        let input = args[2..].join(" ");
        (Some(temp_dir), input, false)
    } else {
        (None, args.join(" "), false)
    }
}

fn render_diagram(input: &str, temp_dir_override: Option<&PathBuf>) -> io::Result<()> {
    let html = build_html(input);

    let dir = get_temp_dir(temp_dir_override.cloned())?;
    let filename = format!(
        "mermaid-{}-{}.html",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let tmp = dir.join(filename);

    fs::write(&tmp, html)?;

    let uri = format!("file:///{}", tmp.display().to_string().replace('\\', "/"));

    if webbrowser::open(&uri).is_err() {
        eprintln!("Failed to open browser. File saved at: {}", tmp.display());
    }

    Ok(())
}

fn print_help() {
    println!(
        r#"mmd-viewer - Instantly render Mermaid diagrams in your browser

USAGE:
    mmd-viewer [OPTIONS] [MERMAID_STRING]
    mmd-viewer [OPTIONS] < stdin

OPTIONS:
    --temp-dir PATH    Override the default temp directory for generated HTML files
    -h, --help         Print this help message

MODES:
    Interactive mode   Run without arguments to enter interactive mode.
                       Type a Mermaid string and press Enter to render it.
                       Type 'exit' or press Ctrl+C to quit.

    CLI mode           Pass a Mermaid string directly as arguments:
                       mmd-viewer "graph TD; A-->B;"

    Pipe mode          Pipe Mermaid content from another command:
                       echo "graph TD; A-->B;" | mmd-viewer
                       cat diagram.mmd | mmd-viewer

EXAMPLES:
    mmd-viewer "graph TD; A-->B; B-->C;"
    mmd-viewer --temp-dir ./output "sequenceDiagram; A->>B: Hello"
    echo "graph LR; A[Login]-->B[Dashboard]; B-->C[Settings];" | mmd-viewer"#
    );
}

fn strip_wrapper_quotes(s: &str) -> &str {
    if s.len() < 2 {
        return s;
    }
    let bytes = s.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    if (first == b'"' && last == b'"')
        || (first == b'\'' && last == b'\'')
        || (first == b'`' && last == b'`')
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn main() -> io::Result<()> {
    let (temp_dir_override, cli_input, show_help) = parse_args();

    if show_help {
        print_help();
        return Ok(());
    }

    if !cli_input.is_empty() {
        return render_diagram(&cli_input, temp_dir_override.as_ref());
    }

    if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.trim().is_empty() {
            eprintln!("Error: no input provided");
            std::process::exit(1);
        }
        return render_diagram(&buf, temp_dir_override.as_ref());
    }

    println!("mmd-viewer - Interactive Mermaid Diagram Renderer");
    println!("Type a Mermaid string and press Enter to render.");
    println!("Type 'exit' or press Ctrl+C to quit.\n");

    let mut line = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;

        line.clear();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            println!("Bye!");
            break;
        }

        let input = strip_wrapper_quotes(trimmed);
        match render_diagram(input, temp_dir_override.as_ref()) {
            Ok(()) => println!("Rendered!\n"),
            Err(e) => eprintln!("Error: {}\n", e),
        }
    }

    Ok(())
}
