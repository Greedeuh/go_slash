function redirectToGoIfGo(tabId, _changeInfo, tab) {
  var url = tab.url;

  if (url.startsWith("http://go/")) {
    chrome.storage.sync.get(
      {
        binding: "",
      },
      function (items) {
        chrome.tabs.update(tabId, {
          url: url.replace("http://go", items.binding),
        });
      }
    );
  }
}
chrome.tabs.onUpdated.addListener(redirectToGoIfGo);
