import './MapView.scss'
import * as hooks from '../../hooks'
import Spinner from "../widgets/Spinner";
import Timestamp from "../widgets/Timestamp";


export default function ListView() {
    const { session } = hooks.useSession();

    if (!session.view) {
        return <Spinner />
    }

    return (

        <div style={{columnCount: 3}}>
            {session.view.refs.map((ref) => (
                <div key={ref.slug} className="ref" style={{breakInside: "avoid-column"}}>
                    <div className="name"><b>{ref.name}</b></div>
                    <div className="lastseen"><Timestamp time={ref.lastseen} /></div>
                    <div className="location">{ref.location}</div>
                </div>
            ))}
        </div>

    );
}
