function drawer(ev) {
  console.log("called")
  const button = ev.currentTarget;
  const icon = button.getElementsByClassName('drawer-icon').item(0);
  const helptext = button.getElementsByClassName('drawer-helptext').item(0);
  const content = button.parentNode?.getElementsByTagName('img').item(0);
  if(icon === null || !helptext === null || content === null) {
    console.log(`${icon}, ${helptext}, ${content}`);
    return;
  }

  if(icon.textContent === '') {
    console.log("Collapsing")
    icon.textContent = '+';
    helptext.textContent = '(Click to expand)';

    content.style.setProperty('max-height', 0);
    content.style.setProperty('padding', 0);
    content.style.setProperty('margin', 0);
    
    content.setAttribute('hidden', 'true');

    button.style.setProperty('border-radius', '1rem')
    button.style.setProperty('border', 'none')
  } else {
    console.log("Expanding")
    icon.textContent = '';
    helptext.textContent = '(Click to collapse)';

    content.style.removeProperty('max-height');
    content.style.removeProperty('padding');
    content.style.removeProperty('margin');

    content.removeAttribute('hidden');

    button.style.removeProperty('border-radius')
    button.style.removeProperty('border')
  }
}
