var ws = null;

function connect() {
  const { location } = window;
  const proto = location.protocol.startsWith("https") ? "wss" : "ws";
  const uri = `${proto}://${location.host}/reload`;
  ws = new WebSocket(uri);

  ws.onmessage = function (e) {
    console.log("message:", e.data);
    location.reload();
  };

  ws.onopen = function () {
    console.log("live reload: connected to websocket");
  };

  ws.onclose = function (e) {
    console.log(
      "socket is closed. reconnect will be attempted in 1 second.",
      e.reason,
    );
    ws = null;

    setTimeout(function () {
      connect();
    }, 100);
  };
  ws.onerror = function (err) {
    console.error("socket encountered error: ", err.message, "closing socket");
  };
}

connect();
