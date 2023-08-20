import './MainMenu.scss'
import {NavLink} from 'react-router-dom';

export default function Menu() {
    return (
        <>
            <NavLink className="mmenu" to="/map">Map</NavLink> 
            <NavLink className="mmenu" to="/list">List</NavLink> 
            <NavLink className="mmenu" to="/log">Log</NavLink>
        </>
    );
}
