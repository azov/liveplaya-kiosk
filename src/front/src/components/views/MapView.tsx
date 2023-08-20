import './MapView.scss'
import * as hooks from '../../hooks'
import type * as api from '../../api'
import Spinner from "../widgets/Spinner";
import Map from '../widgets/Map';


export default function MapView() {
    const [navQuery, navigate] = hooks.useNavQuery();
    const { session, query } = hooks.useSession();
    const onMove = (center: api.LngLat, zoom: number, bounds: api.BBox) => {
        query({ ...session.query, bounds, zoom });
        navigate({ ...navQuery, center: center, zoom });
    }

    if (!session.view) {
        return <Spinner/>
    }

    return (
        <Map data={session.view} className="fill" onMove={onMove}/>
    );
}
