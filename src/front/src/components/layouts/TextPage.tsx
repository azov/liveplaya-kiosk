import './TextPage.scss'
import MainMenu from '../widgets/MainMenu'; 
import { Outlet } from "react-router-dom";

interface Props {
}

export default function TextPage(_p: Props) {
    return (
        <>
            <MainMenu/>
            <div id="text">
                <Outlet/>
            </div>
        </>
    );
}
