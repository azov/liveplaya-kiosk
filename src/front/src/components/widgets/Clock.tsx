interface Props {
    time: string,
    id?: string;
}

export default function Clock(p : Props) {
    return (
        <div id={p.id}>{p.time}</div>
    );
}
