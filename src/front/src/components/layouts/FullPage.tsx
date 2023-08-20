import { Outlet } from "react-router-dom";
import MainMenu from '../widgets/MainMenu'; 


export default function FullPage() {
    return (
        <>
           <header><MainMenu/></header>
           <main className="full"><Outlet/></main>
        </>
    );
}

