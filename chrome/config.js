function save_options() {
  var binding = document.getElementById("binding").value;
  chrome.storage.sync.set(
    {
      binding: binding,
    },
    function () {
      var status = document.getElementById("status");
      status.textContent = "Option saved.";
      setTimeout(function () {
        status.textContent = "";
      }, 750);
    }
  );
}

function restore_options() {
  chrome.storage.sync.get(
    {
      binding: "",
    },
    function (items) {
      document.getElementById("binding").value = items.binding;
    }
  );
}
document.addEventListener("DOMContentLoaded", restore_options);
document.getElementById("save").addEventListener("click", save_options);
