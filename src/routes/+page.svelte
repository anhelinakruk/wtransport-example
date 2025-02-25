<script lang="ts">
  import { onMount } from "svelte";
  let messages: (string | number)[] = [];
  let message: string = '';
  let error: string | null = null;

  async function fetchHello() {
    try {
      console.log("Starting fetch to Axum...");

      const response = await fetch("/wtransport");
      if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
      }
      console.log("Received response:", response);

      const json = await response.json();
      console.log(json);
      } catch (error) {
        console.error(error instanceof Error ? error.message : 'Unknown error occurred');
      }
    }

  async function connectWebTransport() {
      try {

          const HASH = new Uint8Array([189, 52, 188, 150, 143, 93, 146, 176, 158, 71, 97, 191, 150, 43, 177, 159, 110, 94, 178, 64, 87, 161, 218, 249, 127, 134, 80, 135, 196, 123, 149, 9]);
          const url = "https://localhost:4433";
          const transport = new WebTransport(url,  {serverCertificateHashes: [ { algorithm: "sha-256", value: HASH.buffer } ]} );
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
  // onMount(() => {
  //   fetchHello();
  // });
  
</script>


<h1>WebTransport Client</h1>

{#if error}
  <div class="error">
    <p>Error: {error}</p>
    <button on:click={fetchHello}>Retry</button>
  </div>
{:else}
  <div class="messages">
    {#each messages as message}
      <div class="message">{message}</div>
    {/each}
  </div>
  <button on:click={connectWebTransport}>Connect to WebTransport</button>
{/if}

<style>
  .error {
    color: red;
    padding: 1em;
    border: 1px solid red;
    margin: 1em 0;
  }

  .messages {
    margin: 1em 0;
  }

  .message {
    padding: 0.5em;
    margin: 0.5em 0;
    background: #f5f5f5;
    border-radius: 4px;
  }

  button {
    padding: 0.5em 1em;
    margin: 0.5em 0;
    cursor: pointer;
  }
</style>