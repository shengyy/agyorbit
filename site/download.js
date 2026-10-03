// Points the download buttons at the latest release's installers and puts the
// visitor's platform first. Without network access the links keep pointing at
// the releases page.

const REPO = "shengyy/agyorbit";

function preferWindows() {
  const platform = navigator.userAgentData?.platform ?? navigator.platform ?? "";
  return /win/i.test(platform);
}

function setLinks(kind, url) {
  if (!url) return;
  for (const link of document.querySelectorAll(`[data-download="${kind}"]`)) link.href = url;
}

if (preferWindows()) {
  const mac = document.querySelector('.downloads [data-download="mac"]');
  const win = document.querySelector('.downloads [data-download="win"]');
  mac?.classList.remove("primary");
  win?.classList.add("primary");
  win?.parentElement?.prepend(win);
}

fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
  headers: { Accept: "application/vnd.github+json" },
})
  .then((response) => (response.ok ? response.json() : Promise.reject(response.status)))
  .then((release) => {
    const asset = (pattern) => release.assets.find((a) => pattern.test(a.name))?.browser_download_url;
    setLinks("mac", asset(/\.dmg$/i));
    setLinks("win", asset(/setup\.exe$/i) ?? asset(/\.msi$/i));
    for (const label of document.querySelectorAll("[data-version]")) label.textContent = release.tag_name;
  })
  .catch(() => {});
