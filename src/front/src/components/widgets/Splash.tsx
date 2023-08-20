interface Props {
    loading?: boolean,
    children: any,
}

export default function Splash(p: Props) {
    return (
        <div className="splash">
            <p>{p.loading ? "Loading..." : " "}</p>
            {p.children}
        </div>
    );
}
