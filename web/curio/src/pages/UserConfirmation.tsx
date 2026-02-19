import curioLogo from "../assets/curio-logo.png";

const UserConfirmation = () => {
  return (
    <div className="flex flex-col items-center justify-center h-screen ">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="mb-8 w-64 h-auto"
        style={{ mixBlendMode: "lighten" }}
      />
      <div>
        <h1>
          Please check your email for a confirmation link. Once confirmed, you
          will be automatically redirected to home. Thank you for joining Curio,
          hope you enjoy!
        </h1>
      </div>
    </div>
  );
};

export default UserConfirmation;
