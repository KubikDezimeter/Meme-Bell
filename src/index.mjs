import { h, Component, render } from 'https://esm.sh/preact';
import htm from 'https://esm.sh/htm';

// Initialize htm with Preact
const html = htm.bind(h);

function App (props) {
    return html`
        <h1>Hello ${props.name}!</h1>
        <p>This is working with Preact now</p>
    `;
}

render(html`<${App} name="World" />`, document.body);