import {h, render} from 'https://esm.sh/preact';
import htm from 'https://esm.sh/htm';
import {useEffect, useState} from 'https://esm.sh/preact/hooks';
import {signal} from 'https://esm.sh/@preact/signals';

// Initialize htm with Preact
const html = htm.bind(h);

const ringtoneList = signal([]);

async function fetchRingtoneList() {
    const response = await fetch("/api/ringtone_list");
    ringtoneList.value = await response.json();
}
function RingtoneList() {
    useEffect(() => {
        fetchRingtoneList();
    }, []);


    return html`
        <div class="ringtone_table">
        <main class="table">
            <section class="table_header">
                <table>
                    <thead>
                        <tr>
                            <th>#</th>
                            <th>name</th>
                                  <th>sample</th>   
                            <th>status</th>
                            <th></th>
                        </tr>
                    </thead>
                </table>
            </section>
            <section class="table_body">
                <table>
                    <tbody>
                        ${ringtoneList.value.map((ringtone, index) => html`
                            <tr key="${ringtone}">
                                <td class="index_container">${index + 1}</td>
                                <td class="name_container">${ringtone}</td>
                                <td class="audio_container"><audio controls><source src="/api/ringtone/${ringtone}" type="audio/mpeg" />Your browser does not support the audio element.</audio></td>
                                <td class="status_container">
                                    <button class="statusButton" onclick=${toggleStatus} style="border: none;">
                                        <p class="status aktiv">aktiv</p>
                                    </button>
                                </td>
                                <td class="removeButton_container"><${RemoveButton} ringtone="${ringtone}" /></td>
                            </tr>
                        `)}
                    </tbody>
                </table>
            </section>
        </main>
    </div>`;
}
function toggleStatus(event) {
    const statusContainer = event.target.closest('.status_container');
    if (statusContainer) {
        const statusElement = statusContainer.querySelector('.status');
        if (statusElement) {
            if (statusElement.classList.contains('aktiv')) {
                statusElement.textContent = 'passiv';
                statusElement.classList.remove('aktiv');
                statusElement.classList.add('passiv');
            } else {
                statusElement.textContent = 'aktiv';
                statusElement.classList.remove('passiv');
                statusElement.classList.add('aktiv');
            }
        }
    }
}
function RingtoneUploader() {
    const [file, setFile] = useState(null);

    const handleFileChange = async (e) => {
        console.log("handleFileChange started ");
        if (e.target.files && e.target.files.length > 0) {
            console.log("e");
            const selectedFile = e.target.files[0];
            setFile(selectedFile);

            try {
                console.log("handleUpload called");
                await handleUpload(selectedFile);
            } catch (error) {
                console.error(error);
            }
        }
    }

    const handleUpload = async (file) => {
        if (file) {
            console.log("Uploading file...");
            const formData = new FormData();
            formData.append("file", file);
            try {
                const result = await fetch("/api/upload", {
                    method: "POST",
                    body: formData,
                });
                await result;
                ringtoneList.value = [...ringtoneList.value, file.name];
                console.log(ringtoneList.value)
                setFile(null);
            } catch (error) {
                console.error(error);
            }
        }
    };

    return html`
        <section class="upload">
            <div class="file-input">
                <label for="file" class="chooseFileButton">
                    <span>+</span>
                    <input id="file" type="file" accept=".mp3" onChange=${(e) => handleFileChange(e)} />
                </label>
            </div>
        </section>
    `;
}
function RemoveButton(props) {
    const handleRemove = async () => {
        console.log("Moving ringtone " + props.ringtone + " to trash...");

        try {
            await fetch("/api/remove/" + props.ringtone, {
                method: "POST",
            });
            ringtoneList.value = ringtoneList.value.filter((ringtone) => ringtone !== props.ringtone);
            console.log(ringtoneList.value)
        } catch (error) {
            console.error(error);
        }
    }

    return html`<button class="removeButton" type="button" onclick=${handleRemove}>entfernen</button>`;
}
async function fetchRingingTime() {
    const response = await fetch("/api/settings/ringing_time");
    return await response.json();
}
function RingingTimeSetting() {
    const [ringingTime, setRingingTime] = useState();
    useEffect(() => {
        fetchRingingTime().then(setRingingTime);
    }, []);
    const handleChange = async () => {
        let ringingTime = document.getElementById("ringingTime").value;
        try {
            await fetch("/api/settings/ringing_time", {
                method: "PUT",
                body: ringingTime
            });
        } catch (error) {
            console.error(error);
        }
    }

    return html`
        <input class="ringingTime_field" type="number" id="ringingTime" min="1" value="${ringingTime}" oninput="${handleChange}" placeholder="Klingeldauer in Sekunden" />
    `;
}
function toggleComponentsVisibility() {
    const uploader = document.querySelector('.ringtoneUploader');
    const list = document.querySelector('.ringtoneList');

    uploader.classList.toggle('visible');
    list.classList.toggle('visible');
}

function toggleSettingsVisibility() {
    const settingsComponent = document.querySelector('.settingsComponent');
    settingsComponent.classList.toggle('visible');
}

function App() {
    return html`
        <h1 style="padding-bottom: 5px;" id="ringtoneTitle" onclick=${toggleComponentsVisibility}>RINGTONES</h1>
        <div class="ringtoneUploader hidden">${RingtoneUploader()}</div>
        <div class="ringtoneList hidden">${RingtoneList()}</div>
        <h1 style="padding-top: 5px;" onclick=${toggleSettingsVisibility}>SETTINGS</h1>
        <div class="settingsComponent hidden">
            <${RingingTimeSetting} />
        </div>
    `;
}

render(html`<${App} />`, document.body);