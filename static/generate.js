function createImageDropdown(image, categoryName) {
  const container = document.createElement('div');
  container.setAttribute('class', 'container texture-container output-result');

  stateSymbol = document.createElement('span');
  stateSymbol.setAttribute('class', 'drawer-icon fas')
  stateSymbol.textContent = '';

  title = document.createElement('span');
  title.textContent = categoryName
  
  helpText = document.createElement('span');
  helpText.setAttribute('class', 'grow right-align drawer-helptext helptext');
  helpText.textContent = '(click to collapse)'
  
  const headerButton = document.createElement('button');
  headerButton.setAttribute('class', 'header-button');
  headerButton.setAttribute('onclick', 'drawer(event)');
  headerButton.appendChild(stateSymbol);
  headerButton.appendChild(title);
  headerButton.appendChild(helpText);
  
  container.appendChild(headerButton);
  container.appendChild(image);

  return container;
}

function generate(ev) {
  const button = document.getElementById('generate-button');
  button.setAttribute('disabled', 'true');
  button.textContent = 'Generating…'
  
  // Remove all outputs from the output pane
  let outputs = document.getElementsByClassName('output-result')
  while(outputs[0] !== undefined) {
    outputs[0].remove()
  }


  let outputPane = document.getElementById('output');

  const script = document.getElementById('editor').value;
  // The following fetch was heavily  adapted from the
  // highest rated answer here (the one by maxpoj on May 9, 2018).
  // https://stackoverflow.com/questions/50248329/fetch-image-from-api
  // Referenced on Mon 25 Apr
  //
  // The key was URL.createObjectURL, I had no idea how to get the
  // data in memory as an url that the <img> tag could use, but
  // createObjectURL allows me to create an url to the data
  // returned by the fetch.
  fetch('generate', {
    method: 'POST',
    headers: {
      'Cache-Control': 'no-cache',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ cityscript: script }),
  }).then(async (response) => {
    if(response.status > 199 && response.status < 300) {
      return response.blob();
    } else if(response.status > 399 && response.status < 500) {
      let json = await response.json();
      if(json.error !== undefined) {
        throw Error(`${json.error}`);
      } else {
        throw Error('Bad script input');
      }
    } else {
      let json = await response.json();
      if(json.error !== undefined) {
        throw Error(`${json.error}`);
      } else {
        throw Error('Service unavailable');
      }
    }
  }).then((blob) => {
    const url = URL.createObjectURL(blob);
    const image = document.createElement('img');

    image.setAttribute('class', 'texture');
    image.setAttribute('src', url);
    outputPane.appendChild(createImageDropdown(image, "Albedo"));
  }).catch((error) => {
    // "Failed to obtain image from the service, is it down?";
    
    const errorMessage = document.createElement('p');
    errorMessage.textContent = error.toString();
    errorMessage.setAttribute('class', 'output-result error-message');
    outputPane.appendChild(errorMessage);
  }).finally(() => {

    button.removeAttribute('disabled');
    buttonIcon = document.createElement('span');
    buttonIcon.setAttribute('class', 'fas')
    buttonIcon.textContent = '';

    button.textContent = 'Generate ';
    button.appendChild(buttonIcon);
  });
}


function handleKeydown(ev) {
  console.log(ev);
  if (ev.ctrlKey && ev.key === 'Enter') {
    generate(ev);
  }
}

let body = document.getElementById("body");
let button = document.getElementById("generate-button");

body.onkeydown = handleKeydown;
button.onclick = generate;
