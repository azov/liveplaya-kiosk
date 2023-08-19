import './MainMenu.scss'
import {NavLink} from 'react-router-dom';

// interface Link {
//     url: string,
//     name: string,
// }

// interface Props {
//     links: Link[],
//     id?: string;
// }

export default function Menu() {
    return (
        <nav id="mainmenu">
            <NavLink to="/map">Map</NavLink> 
            <NavLink to="/log">Log</NavLink>
        </nav>
    );
}
