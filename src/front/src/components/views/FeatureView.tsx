import './FeatureView.scss'
import * as hooks from '../../hooks'
import Splash from "../widgets/Splash";
import Timestamp from "../widgets/Timestamp";

export default function MapFeatureView() {
    const { session } = hooks.useSession();

    if (!session.view) {
        return <Splash loading={session.isLoading}>
        </Splash>
    }

    const page = session.view;

    return (
        <div className="mapfeature">
            <h1>{page.name}</h1>
            <div className="refs">
                {page.refs.map((ref) => (
                    <div key={ref.slug} className="ref">
                        <div className="name">{ref.name}</div>
                        <div className="lastseen"><Timestamp time={ref.lastseen}/></div>
                        <div className="location">{ref.location}</div>
                    </div>
                ))}
            </div>
        </div>
    );
}
