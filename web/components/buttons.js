/**
 * Adds the buttons CSS to current page.
 */
function add_buttons() {
    // Add style to header
    const head = document.querySelector("head");
    if (!head) {
        console.error("Could not get head");
    }
    const link = document.createElement("link");
    link.href = "/styles/buttons.css";
    link.rel = "stylesheet";
    head.appendChild(link);
}

add_buttons();