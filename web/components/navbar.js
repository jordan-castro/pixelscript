const NAVBAR = `
<nav class="pxs-navbar-style navbar navbar-expand-lg bg-body-tertiary">
  <div class="container-fluid">
    <a class="navbar-brand" href="#">Pixelscript</a>
    <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation">
      <span class="navbar-toggler-icon"></span>
    </button>
    <div class="collapse navbar-collapse" id="navbarSupportedContent">
      <ul class="navbar-nav me-auto mb-2 mb-lg-0">
        <li class="nav-item">
          <a class="nav-link active" aria-current="page" href="/">Home</a>
        </li>
        <li class="nav-item">
          <a class="nav-link" href="/playground.html">Playground</a>
        </li>
        <li class="nav-item dropdown">
          <a class="nav-link dropdown-toggle" href="#" role="button" data-bs-toggle="dropdown" aria-expanded="false">
            Docs
          </a>
          <ul class="dropdown-menu">
            <li><a class="dropdown-item" href="/docs">Getting Started</a></li>
            <li><a class="dropdown-item" href="/docs/corelib.html">CoreLib</a></li>
          </ul>
        </li>
      </ul>
      {{form}}
    </div>
  </div>
</nav>
`;

/**
 * Adds the navbar to current page.
 */
function add_navbar() {
    const nav = document.getElementById("pxs-navbar");
    if (!nav) {
        console.error("Could not get navbar.");
        return;
    }

    let navbar = NAVBAR;
    if (window.location.toString().includes("/docs")) {
        navbar = navbar.replace("{{form}}", `
            <form class="d-flex" role="search">
                <input class="form-control me-2" type="search" placeholder="Search" aria-label="Search"/>
                <button class="btn btn-outline-success" type="submit">Search</button>
            </form>
        `);
    } else {
        navbar = navbar.replace("{{form}}", "");
    }

    nav.innerHTML = navbar;

    // Add style to header
    const head = document.querySelector("head");
    if (!head) {
        console.error("Could not get head");
    }
    head.innerHTML += '<link rel="stylesheet" href="/styles/navbar.css" />';
}

add_navbar();