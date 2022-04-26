function generate(ev) {
  let images = document.getElementsByClassName('texture')
  let rem = images[0]
  while(rem !== undefined) {
    rem.remove()
    rem = images[0]
  }
  
  let output = document.getElementById('output');
  let image = document.createElement('img');

  image.setAttribute('class', 'texture');

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
    .then((response) => response.blob())
    .then((blob) => {
      const url = URL.createObjectURL(blob)
      image.setAttribute('src', url);
      output.appendChild(image);
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
