import React from 'react';
import './PenguinWalking.css';

interface PenguinWalkingProps {
  duration?: number; // Duration in seconds for the penguin to walk across the screen
}

const PenguinSvg = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="80"
    height="100"
    viewBox="0 0 80 100"
  >
    <g fill="none" fillRule="evenodd">
      {/* Body */}
      <ellipse
        className="penguin-body"
        cx="40"
        cy="55"
        rx="25"
        ry="30"
        fill="#000000"
        stroke="#1a1a1a"
        strokeWidth="2"
      />

      {/* White belly */}
      <ellipse
        cx="40"
        cy="58"
        rx="16"
        ry="22"
        fill="#FFFFFF"
      />

      {/* Head */}
      <circle
        cx="40"
        cy="25"
        r="18"
        fill="#000000"
        stroke="#1a1a1a"
        strokeWidth="2"
      />

      {/* Left eye white */}
      <ellipse
        cx="33"
        cy="23"
        rx="5"
        ry="6"
        fill="#FFFFFF"
      />

      {/* Right eye white */}
      <ellipse
        cx="47"
        cy="23"
        rx="5"
        ry="6"
        fill="#FFFFFF"
      />

      {/* Left eye pupil */}
      <circle
        className="left-eye-pupil"
        cx="33"
        cy="24"
        r="2.5"
        fill="#000000"
      />

      {/* Right eye pupil */}
      <circle
        className="right-eye-pupil"
        cx="47"
        cy="24"
        r="2.5"
        fill="#000000"
      />

      {/* Beak */}
      <path
        d="M 40 28 L 35 32 L 45 32 Z"
        fill="#FFA500"
        stroke="#FF8C00"
        strokeWidth="1"
      />

      {/* Left wing */}
      <ellipse
        className="left-wing"
        cx="20"
        cy="55"
        rx="8"
        ry="20"
        fill="#1a1a1a"
        stroke="#000000"
        strokeWidth="1.5"
        transform="rotate(-20 20 55)"
      />

      {/* Right wing */}
      <ellipse
        className="right-wing"
        cx="60"
        cy="55"
        rx="8"
        ry="20"
        fill="#1a1a1a"
        stroke="#000000"
        strokeWidth="1.5"
        transform="rotate(20 60 55)"
      />

      {/* Left foot */}
      <ellipse
        className="left-foot"
        cx="30"
        cy="88"
        rx="8"
        ry="5"
        fill="#FFA500"
        stroke="#FF8C00"
        strokeWidth="1.5"
      />

      {/* Right foot */}
      <ellipse
        className="right-foot"
        cx="50"
        cy="88"
        rx="8"
        ry="5"
        fill="#FFA500"
        stroke="#FF8C00"
        strokeWidth="1.5"
      />
    </g>
  </svg>
);

const PenguinWalking: React.FC<PenguinWalkingProps> = ({ duration = 8 }) => {
  return (
    <div className="penguin-container">
      <div
        className="penguin-walker"
        style={{
          animationDuration: `${duration}s`
        }}
      >
        <PenguinSvg />
      </div>
    </div>
  );
};

export default PenguinWalking;
