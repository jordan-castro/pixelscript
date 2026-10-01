/**
 * Adds the bootstrap to current page.
 */
function add_bootstrap() {
    // Add style to header
    const head = document.querySelector("head");
    if (!head) {
        console.error("Could not get head");
    }
    const link = document.createElement("link");
    link.href = "https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/css/bootstrap.min.css";
    link.rel = "stylesheet";
    link.integrity = "sha384-sRIl4kxILFvY47J16cr9ZwB07vP4J8+LH7qKQnuqkuIAvNWLzeN8tE5YBujZqJLB";
    link.crossOrigin = "anonymous";
    head.appendChild(link);
    const script = document.createElement("script");
    script.src = "https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/js/bootstrap.bundle.min.js";
    script.integrity = "sha384-FKyoEForCGlyvwx9Hj09JcYn3nv7wiPVlz7YYwJrWVcXK/BmnVDxM+D2scQbITxI";
    script.crossOrigin = "anonymous";
    head.appendChild(script);
}

add_bootstrap();