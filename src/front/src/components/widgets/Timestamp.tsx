interface Props {
    time: string | Date,
    // style: "abs"|"rel",
}

export default function Timestamp(p: Props) {
    let t = p.time;
    if (!(t instanceof Date)) {
        t = new Date(t);
    }

    return (
        <span>{daysAgoStr(t)}</span>
    );
}


function dateStr(then: any) {
    return then.getMonth() + '/' + then.getDay() + '/' + then.getFullYear() + ' ' +
        (then.getHours() % 12) + ':' + then.getMinutes() + (then.getHours() < 12 ? 'am' : 'pm');
}

function daysAgoStr(then: Date | null) {
    if (then === null || then === undefined) {
        return then;
    }

    var diff = Math.round((Date.now() - then.getTime()) / 1000);
    if (diff < 0) {
        return 'on ' + dateStr(then);
    }

    var second_diff = diff % 86400;
    var day_diff = Math.round(diff / 86400);

    if (day_diff == 0) {
        if (second_diff < 10) return "now";
        //if (second_diff < 60) return second_diff + " seconds ago";
        if (second_diff < 120) return "a minute ago";
        if (second_diff < 3600) return Math.round(second_diff / 60) + " minutes ago";
        if (second_diff < 7200) return "an hour ago";
        if (second_diff < 86400) return Math.round(second_diff / 3600) + " hours ago";
    }
    if (day_diff == 1) return "yesterday";
    if (day_diff < 7) return day_diff + " days ago";
    return 'on ' + dateStr(then);
    // if (day_diff < 14) return "last week";
    // if (day_diff < 31) return Math.round(day_diff/7) + " weeks ago";
    // if (day_diff < 60) return "last month";
    // if (day_diff < 365) return Math.round(day_diff/30) + " months ago";
    // if (day_diff < 730) return "a year ago";
    // return Math.round(day_diff/365) + " years ago";
}