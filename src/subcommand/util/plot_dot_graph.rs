use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn plot_dot_graph(dot_graph_text: &String, format: &'static str, original_file: &Path) {
    let path_to_render = original_file.with_extension(format);
    let mut child = Command::new("dot") // Use `dot` layout engine
        .arg(format!("-T{}", format))
        .arg("-o")
        .arg(path_to_render.as_os_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn \"dot\" (graphviz)");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(dot_graph_text.as_bytes())
            .expect("Failed to write to stdin");
        // Drop `stdin` to close stdin
    }

    let _ = child.wait_with_output().expect("Failed to read stdout");
    log::info!("Rendered to file \"{}\"", path_to_render.display());
}
