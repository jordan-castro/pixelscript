/**
 * Adds the bootstrap to current page.
 */
function add_tables_style() {
    // Add style to header
    const head = document.querySelector("head");
    if (!head) {
        console.error("Could not get head");
    }
    const link = document.createElement("link");
    link.href = "/styles/tables.css";
    link.rel = "stylesheet";
    head.appendChild(link);
}

add_tables_style();