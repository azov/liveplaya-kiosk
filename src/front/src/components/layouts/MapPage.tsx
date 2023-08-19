import './MapPage.scss'
import * as hooks from '../../hooks'
import type * as api from '../../api'
import { Outlet } from "react-router-dom";
import Map from '../widgets/Map';
import MainMenu from '../widgets/MainMenu'; 
import Splash from "../widgets/Splash";


export default function MapPage() {
    const [navQuery, navigate] = hooks.useNavQuery();
    const { session, query } = hooks.useSession();
    const onMove = (center: api.LngLat, zoom: number, bounds: api.BBox) => {
        query({ ...session.query, bounds, zoom });
        navigate({ ...navQuery, center: center, zoom });
    }

    if (!session.view) {
        return <Splash loading={session.isLoading}>
        </Splash>
    }

    const view = session.view;

    return (
        <>
           <MainMenu/>
           <Map id="map" data={view} onMove={onMove}/>
            <div id="mapoverlay">
                <Outlet/>
            </div>
        </>
    );
}

