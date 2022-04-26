function generate(ev) {
  const button = document.getElementById('generate-button');
  button.setAttribute('disabled', 'true');
  button.textContent = "Generating…"
  
  // Remove all outputs from the output pane
  let outputs = document.getElementsByClassName('output-result')
  while(outputs[0] !== undefined) {
    outputs[0].remove()
  }

  let outputPane = document.getElementById('output');
  // The following fetch was adapted from the highest rated answer here
  // (the one by maxpoj on May 9, 2018).
  // Referenced on Mon 25 Apr
  // https://stackoverflow.com/questions/50248329/fetch-image-from-api
  //
  // The key was URL.createObjectURL, I had no idea how to get the
  // data in memory as an url that the <img> tag could use, but
  // createObjectURL allows me to create an url to the data
  // returned by the fetch.
  fetch('generate', {headers: {'Cache-Control': 'no-cache'}})
    .then((response) => {
      switch(response.status) {
        case 200:
          return response.blob()
        case 503:
          throw Error("Service unavailable")
      }
    })
    .then((blob) => {
      const url = URL.createObjectURL(blob)
      const image = document.createElement('img');
      image.setAttribute('class', 'texture output-result');
      image.setAttribute('src', url);
      outputPane.appendChild(image);
    })
    .catch((error) => {
      const errorMessage = document.createElement('p');
      errorMessage.textContent = error.toString(); // "Failed to obtain image from the service, is it down?";
      errorMessage.setAttribute('class', 'error-message output-result');
      outputPane.appendChild(errorMessage);
    })
    .finally(() => {
      button.removeAttribute('disabled');
      button.textContent = "Generate"
    });
}


function handleKeydown(ev) {
  console.log(ev);
  if (ev.key === 'g') {
    generate(ev);
  }
}


let body = document.getElementById("body");
let button = document.getElementById("generate-button");

body.onkeydown = handleKeydown;
button.onclick = generate;
