import * as hooks from '../../hooks'

export default function Spinner() {
    const { session: state } = hooks.useSession();

    return state.isLoading ? (
        <div id="spinner">Loading</div>) : (<></>);
}
