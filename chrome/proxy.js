function redirectToGoIfGo(tabId, _changeInfo, tab) {
  var url = tab.url;

  if (url.startsWith("http://go/")) {
    chrome.tabs.update(tabId, {
      url: url.replace("http://go/", "http://go.olivon.fr/"),
    });
  }
}
chrome.tabs.onUpdated.addListener(redirectToGoIfGo);
