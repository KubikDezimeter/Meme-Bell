import { h, Component, render } from 'https://esm.sh/preact';
import htm from 'https://esm.sh/htm';
import { useState, useEffect } from 'https://esm.sh/preact/hooks';


// Initialize htm with Preact
const html = htm.bind(h);

// document.addEventListener("click", () => location.reload())

function RingtoneList() {
    const [ringtoneList, setRingtoneList] = useState([]);

    useEffect(async () => {
        await fetch("/api/get_ringtone_list")
            .then((response) => response.json())
            .then(setRingtoneList);
    }, []);

    return html`
    <table>
        <tr>
            <th>Klingelton aktiv</th>
            <th>Name</th>
            <th></th>
            <th></th>
        </tr>
        ${ringtoneList.map((ringtone) => html`
        <tr>
            <td><input type="checkbox" name="${ringtone}" /></td>
            <td>${ringtone}</td>
            <td><audio controls><source src="/api/get_ringtone/${ringtone}" type="audio/mpeg" />Your browser does not support the audio element.</audio></td>
            <td><button type="button">Entfernen</button></td>
        </tr>`)}
    </table>`;
}

function App (props) {
    return html`
        <h1>Meme Klingel</h1>
        <${RingtoneList} />
    `;
}

render(html`<${App} name="World" />`, document.body);