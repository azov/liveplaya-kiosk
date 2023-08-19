import Logo from './Logo';

interface Props {
    loading?: boolean,
    children: any,
}

export default function Splash(p: Props) {
    return (
        <div className="splash">
            <Logo size="large"/>
            <p>{p.loading ? "Loading..." : " "}</p>
            {p.children}
        </div>
    );
}
