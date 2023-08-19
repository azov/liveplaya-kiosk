import './LogView.scss'
import * as hooks from '../../hooks'
import Splash from "../widgets/Splash";


export default function LogView() {
    const { session } = hooks.useSession();

    if (!session.view?.log) {
        return <Splash loading={session.isLoading}>
        </Splash>
    }

    const log = session.view.log;

    return (
        <table className="log">
            <tbody>
                {log.map((entry) => (
                    <tr key={entry.id} className={entry.level}><td>{entry.time}</td><td>{entry.text}</td></tr>
                ))}
            </tbody>
        </table>
    );
}
