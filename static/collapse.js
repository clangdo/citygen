function textureDrawer(ev) {
  const button = ev.target;
  const icon = button.getElementsByClassName('drawer-icon').item(0);
  const helptext = button.getElementsByClassName('drawer-helptext').item(0);
  const content = button.parentNode?.getElementsByTagName('img').item(0);
  if(icon === undefined || helptext === undefined || content === undefined) {
    console.log(`{icon}, {helptext}, {content}`);
    return;
  }

  if(icon.textContent === '') {
    console.log("Collapsing")
    icon.textContent = '+';
    helptext.textContent = '(Click to expand)';
    content.style.setProperty('max-height', 0);
  } else {
    console.log("Expanding")
    icon.textContent = '';
    helptext.textContent = '(Click to collapse)';
    content.style.removeProperty('max-height');
  }
}
