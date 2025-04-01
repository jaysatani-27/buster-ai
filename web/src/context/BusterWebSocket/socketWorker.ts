self.onmessage = function (e: MessageEvent) {
  const message = e.data;

  try {
    const data = JSON.parse(message);
    self.postMessage({ type: 'parsed', data, ogData: message });
  } catch (error) {
    self.postMessage({ type: 'error', error });
  }
};
