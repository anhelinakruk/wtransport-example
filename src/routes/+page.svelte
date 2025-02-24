<script lang="ts">
  let messages: (string | number)[] = [];

  async function connectWebTransport() {
      try {
          const transport = new WebTransport("https://anhelina.visoft.dev/wtransport");
          await transport.ready;

          console.log("Connected to WebTransport server");

          const uds = transport.incomingUnidirectionalStreams;
          const reader = uds.getReader();

          console.log(uds); 
          console.log(reader);

          while (true) {
            const { done, value } = await reader.read();
            if (done) {
              break;
            }
            // value is an instance of WebTransportReceiveStream
            await readData(value);
          }
      } catch (error) {
          console.error("WebTransport connection error:", error);
      }
  }

  async function readData(receiveStream: any) {
    const reader = receiveStream.getReader();
    while (true) {
        const { done, value } = await reader.read();
        if (done) {
            break;
        }

        const text = new TextDecoder().decode(value);
        
        const number = parseInt(text, 10);
        if (!isNaN(number)) {
            console.log("Number:", number);
            messages = [...messages, number];
        } else {
            console.warn("Failed to parse number:", text);
        }
    }
}

  connectWebTransport();
</script>

<h1>WebTransport Client</h1>
<ul>
  {#each messages as message}
      <li>{message}</li>
  {/each}
</ul>
