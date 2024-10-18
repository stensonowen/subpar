
var MAP = null;
var RESP = null;
var ICON = {};
var UPCOMING_CELLS = new Array();
const API_URL = 'https://api.subpar.nyc';
// const API_URL = 'http://localhost:3000';

function get_complex_id() {
    const urlparts = document.location.pathname.split('/');
    const idx = urlparts.indexOf('c');
    if (idx > -1 && idx < urlparts.length) {
        return urlparts[idx+1];
    }
    const re = /id=(\d+)/;
    const m = document.location.search.match(re);
    if (m[1]) {
        return m[1];
    }
    throw new Error(`no complex id found in "${document.location}"`);
}

function paintmap() {
    // todo maybe don't paint the map until we know the point to center on
    // to reduce unnecessary tile queries
    // the center-to animation is cool though
    MAP = L.map('map', {attributionControl: false}).setView([40.752, -73.98], 15);
    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {}).addTo(MAP);
}

function get_info() {
    const cid = get_complex_id();
    const url = `${API_URL}/complex/${cid}`;
    console.log(`querying ${url}`);
    fetch(url).then(rsp => rsp.json().then(on_complex_response)).catch(e => console.error(e));
}
function on_complex_response(json) { // api::ComplexInfo
    RESP = json;
    console.log(json);

    document.getElementById('cplx_name').innerHTML = json.meta.name + '&nbsp;';
    let ada = "ADA Compliance: " + json.meta.ada;
    if (json.meta.ada_notes) {
        ada += ". " + json.meta.ada_notes;
    }
    document.getElementById('cplx_ada').innerHTML = ada;
    const routeParent = document.getElementById('cplx_name');
    for (const route of json.meta.routes) {
        routeParent.appendChild(make_bullet(route));
    }
    document.title = 'Subpar | ' + json.meta.name;

    L.marker(json.meta.coord, {icon: ICON.marker})
        .addTo(MAP)
        .bindPopup(json.meta.stop_name);
    MAP.flyTo(json.meta.coord, 17);
    for( entr of json.meta.entrances ) {
        const opts = {
            'Elevator': { icon: ICON.elevator, zIndexOffset: 1000 },
            'Stair/Escalator': { icon: ICON.escalator },
            'Easement - Street': { icon: ICON.escalator },
            'Easement - Passage': { icon: ICON.escalator },
            'Escalator': { icon: ICON.escalator },
            'Stair': { icon: ICON.stair },
        }[entr.entrance_type];
        if (!opts) { continue }
        L.marker([entr.entrance_latitude, entr.entrance_longitude], opts)
            .addTo(MAP)
            .bindPopup('"' + entr.entrance_type + '" to ' + entr.daytime_routes.join(', '));
    }

    paint_upcoming(json.upcoming);
    paint_elevators2(json.elevators);

    setInterval(refetch_upcoming, 30*1000);
}

function refetch_upcoming() {
    const cid = get_complex_id();
    const url = `${API_URL}/upcoming/${cid}`;
    console.log(`fetching upcoming ${url}`);
    fetch(url).then(rsp => rsp.json().then(paint_upcoming)).catch(e => console.error(e));
}

function paint_upcoming(trains) {
    console.log('refreshing');
    trains.sort( (a,b) => a.arrival > b.arrival );
    let routes = new Set(); // insertion order
    const re = /_(\w)X?..(N|S)/;

    const tableN = document.getElementById('upcoming-body-N');
    const tableS = document.getElementById('upcoming-body-S');
    UPCOMING_CELLS = new Array();
    const now = new Date();
    let max_upd = new Date(0);
    tableN.innerHTML = '<thead> <tr> <th> N/E </th> <th> Minutes </th> </tr> </thead>';
    tableS.innerHTML = '<thead> <tr> <th> S/W </th> <th> Minutes </th> </tr> </thead>';
    for ( const train of trains ) {
        const arrival = new Date(train.arrival);
        const delta_ms = arrival - now;
        if (delta_ms < 0) {
            console.trace('skipping negative arrival ', train);
            continue
        }
        const [_, route, dir] = train.trip.match(re);

        const rowElem = document.createElement('tr');
        const c1 = document.createElement('th');
        c1.appendChild(make_bullet(route));
        let c2 = document.createElement('td');
        c2.arrival = arrival;
        c2.message = new Date(train.message);
        max_upd = Math.max(max_upd, c2.message);
        UPCOMING_CELLS.push(c2);

        rowElem.appendChild(c1);
        rowElem.appendChild(c2);
        if (dir == 'N') {
            tableN.appendChild(rowElem);
        } else {
            tableS.appendChild(rowElem);
        }
    }
    document.getElementById('upcoming-details').open = true;
    let timer1 = document.getElementById('upcoming-timer-container');
    timer1.innerHTML = '';

    let timer2 = document.createElement('span');
    timer2.classList.add('aging-effect');
    timer2.innerText = "AAAAAAAA";
    timer2.timestamp = max_upd;

    const seconds_ago = Math.round((now - max_upd) / 1000);
    if (seconds_ago > 0) {
        timer2.style['animation-delay'] = `-${seconds_ago}s`;
    }
    timer1.appendChild(timer2);

    refresh_upcoming_etas();
}

function refresh_upcoming_etas() {
    const now = new Date();
    for ( let cell of UPCOMING_CELLS ) {
        const m = Math.round((cell.arrival - now) / 60 / 100) / 10;
        cell.innerHTML = m;
        if (m < 0) {
            if ( cell.classList.contains('train_missed') === false ) {
                cell.classList.add('train_missed');
            }
        }
    }
    const time = document.querySelector('#upcoming-timer-container span');
    const age = Math.round((now - time.timestamp) / 1000);
    time.innerText = `As of ${age}s ago`;
    setTimeout( refresh_upcoming_etas, 5000 );
}

function paint_elevators2(elevators) {
    const table = document.getElementById('elevator-list');
    elevators.sort( (a,b) => a.is_escalator > b.is_escalator );
    let asof = null;
    for (const elev of elevators) {
        if (elev.outage) {
            asof = Math.max(elev.outage.asof);
        }
        let msgElem = document.createElement('article');
        msgElem.classList.add('message');

        let icon = elev.is_escalator ? '/img/escalator2.svg' : '/img/elevator4.svg';
        let label = elev.is_escalator ? 'Escalator' : 'Elevator';
        let active = elev.is_active ? '\u2705 Active' : '\u274C Inactive';
        let ada = elev.ada ? '\u2705 ADA' : '\u274C ADA';
        let outage = elev.outage ? '\u274C Outage' : '';

        let headerElem = document.createElement('div');
        headerElem.classList.add('message-header');
        headerElem.innerHTML = `
            <img class="icon" src="${icon}" > 
            <span> ${label} </span>
            <span> ${outage} &nbsp; ${ada} &nbsp; ${active} </span>
        `;

        let bodyElem = document.createElement('div');
        bodyElem.classList.add('message-body');
        const out = elev.outage;
        if (out) {
            const fmt = s => s.substr( 0, s.indexOf('T') );
            const ada = out.ada ? 'ADA \u2705' : 'ADA \u274C';
            bodyElem.innerHTML = `
                <p> ${elev.serving} </p>
                <p> <i>Alternative</i>: ${elev.alt_desc} </p>
                <p> <i>Buses</i>: ${elev.buses} </p>
                <p> Starting ${fmt(out.start)} until ${fmt(out.est_return)} </p>
                <p> ${ada}. <i>Reason</i>: ${out.reason} </p>
            `;
        } else {
            bodyElem.innerHTML = `<p> ${elev.serving} </p>`;
        }
        msgElem.appendChild(headerElem);
        msgElem.appendChild(bodyElem);
        table.appendChild(msgElem);
    }
    const time = document.getElementById('elevator-preview-t');
    if (asof) {
        const age = Math.round((new Date() - asof) / 1000 / 60);
        time.innerHTML = `As of ${age} mins ago`;
        time.style.animationDelay = `-${age}s`;
    }
}


function paint_elevators(elevators) {
    const table = document.getElementById('elevator-table');
    function append(k, v) {
        let row = document.createElement('tr');
        let c1 = document.createElement('td');
        c1.innerText = k;
        let c2 = document.createElement('td');
        c2.innerText = v;
        row.appendChild(c1);
        row.appendChild(c2);
        table.appendChild(row);
    }
    for (const elevator of elevators) {
        append('', elevator['is_escalator'] ? 'Escalator' : 'Elevator');
        for (const key of [
            'serving',
            'ada',
            'is_escalator',
            'is_active',
            'buses',
            'outage',
        ]) {
            append(key, elevator[key]);
        }
        break
    }
}

function make_bullet(route_char) {
    let elem = document.createElement('img');
    elem.classList.add('bullet');
    elem.src = `/img/R${route_char}.svg`;
    elem.alt = route_char;
    return elem;
}

function init_icons() {
    ICON.elevator = L.icon({
        iconUrl: '/img/elevator4.svg',
        iconSize: [50, 50],
        iconAnchor: [25, 50],
    });
    ICON.stair = L.icon({
        iconUrl: '/img/stairs3.svg',
        iconSize: [40, 40],
        iconAnchor: [20, 40],
    });
    ICON.marker = L.icon({
        iconUrl: '/img/star1.svg',
        iconSize: [35, 35],
    });
    ICON.escalator = L.icon({
        iconUrl: '/img/escalator2.svg',
        iconSize: [40, 40],
        iconAnchor: [20, 40],
    });

}



function main() {
    init_icons();
    paintmap();
    get_info();
    console.log('ok');
}
